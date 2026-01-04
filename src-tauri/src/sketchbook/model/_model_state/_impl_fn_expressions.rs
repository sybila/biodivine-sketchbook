use std::collections::{HashMap, HashSet};

use crate::sketchbook::ids::UninterpretedFnId;
use crate::sketchbook::model::{FnTree, ModelState, UninterpretedFn, UpdateFn};

impl ModelState {
    /// If the given update function's expression uses any other function symbols that have
    /// their expressions specified, substitute these symbols with the provided expressions.
    ///
    /// We substitute the function symbols using the function expressions provided in the
    /// `expression_trees` mapping. The symbols with empty expressions are expected to
    /// be mapped to `None`. This set must contain all the referenced functions.
    ///
    /// For example, if the update function has an expression `f_A = g(B) | C`, and
    /// we have `g(x1) = !x1`, then this method will substitute `g(B)` with `!B`, resulting
    /// in `f_A = !B | C`.
    pub fn substitute_expressions_to_update_fn(
        self: &ModelState,
        update_fn: &UpdateFn,
        expression_trees: &HashMap<UninterpretedFnId, Option<FnTree>>,
    ) -> Result<FnTree, String> {
        let mut update_fn_tree = update_fn.get_fn_tree().clone().unwrap();
        // check if this update fn contains any fn symbols that have their expressions specified
        let fn_symbols_used = update_fn.collect_fn_symbols();
        for fn_id in fn_symbols_used {
            if let Some(optional_tree) = expression_trees.get(&fn_id) {
                if let Some(fn_expression_tree) = optional_tree {
                    // There is an expression and we substitute the function symbol (inside of the
                    // update fn tree) with that expression.
                    // All the magical transformations happen inside this method.
                    update_fn_tree = update_fn_tree
                        .substitute_fn_symbol_with_expression(&fn_id, fn_expression_tree);
                } else {
                    // This function symbol has empty expression, so we simply leave it there.
                    continue;
                }
            } else {
                return Err(format!(
                    "Expression for substitution of function {fn_id} not provided."
                ));
            }
        }
        Ok(update_fn_tree)
    }

    /// If the given function's expression uses any other function symbols that have their
    /// expressions specified, substitute these symbols with the provided expressions.
    ///
    /// We substitute the function symbols using the function expressions provided in the
    /// `expression_trees` mapping. The symbols with empty expressions are expected to
    /// be mapped to `None`. This set must contain all the referenced functions.
    ///
    /// For example, if the function has an expression like `f(x, y) = x | g(y) & h(y)`, and
    /// we know `g(x) = !x` and `h` is unspecified, then this method will substitute `g(y)`
    /// with `!y`, resulting in `f(x, y) = x | !y & h(y)`.
    pub fn substitute_expressions_to_uninterpreted_fn(
        self: &ModelState,
        uninterpreted_fn: &UninterpretedFn,
        expression_trees: &HashMap<UninterpretedFnId, Option<FnTree>>,
    ) -> Result<FnTree, String> {
        let mut fn_tree = uninterpreted_fn.get_fn_tree().clone().unwrap();
        // check if this update fn contains any fn symbols that have their expressions specified
        let fn_symbols_used = uninterpreted_fn.collect_fn_symbols();
        for fn_id in fn_symbols_used {
            if let Some(optional_tree) = expression_trees.get(&fn_id) {
                if let Some(fn_expression_tree) = optional_tree {
                    // There is an expression and we substitute the function symbol (inside of the
                    // update fn tree) with that expression.
                    // All the magical transformations happen inside this method.
                    fn_tree =
                        fn_tree.substitute_fn_symbol_with_expression(&fn_id, fn_expression_tree);
                } else {
                    // This function symbol has empty expression, so we simply leave it there.
                    continue;
                }
            } else {
                return Err(format!(
                    "Expression for substitution of function {fn_id} not provided."
                ));
            }
        }
        Ok(fn_tree)
    }

    /// This method will substitute expressions of all function symbols that
    /// are used in expressions of other uninterpreted functions. The substitution
    /// is done recursively, so that all expressions are fully propagated.
    ///
    /// Method returns mapping from function IDs to their new expression trees
    /// (or None if they don't have an expression to start with).
    ///
    /// There must be no cycles in the expressions of uninterpreted functions,
    /// otherwise this method will return error.
    ///
    /// For example, if we have three functions with the following expressions:
    /// - `f(x, y) = g(x) | h(x, y)`
    /// - `g(x) = !h(x, x)`
    /// - `h(x, y) = x & y`
    ///
    /// The method will transform the expressions to:
    /// - `f(x, y) = (!(x & x)) | (x & y)`
    /// - `g(x) = !(x & x)`
    /// - `h(x, y) = x & y`
    pub fn propagate_expressions_through_uninterpreted_fns(
        &self,
    ) -> Result<HashMap<UninterpretedFnId, Option<FnTree>>, String> {
        // Assert there are no cycle references (so that the algo terminates)
        self.assert_no_cycles_in_fn_expressions()?;

        // We will propagate the expressions step by step.
        // We start with a set of functions that have empty expressions or do not reference any
        // other functions in their expressions.
        let mut remaining_functions: HashMap<UninterpretedFnId, UninterpretedFn> = HashMap::new();
        let mut processed_functions: HashMap<UninterpretedFnId, Option<FnTree>> = HashMap::new();
        for (fn_id, uninterpreted_fn) in self.uninterpreted_fns() {
            if uninterpreted_fn.has_empty_expression() {
                // atomic case 1 - no expression at all
                processed_functions.insert(fn_id.clone(), None);
            } else if uninterpreted_fn.collect_fn_symbols().is_empty() {
                // atomic case 2 - no fn symbols to substitute, so the expression stays as it is
                let fn_tree = uninterpreted_fn.get_fn_tree().clone().unwrap();
                processed_functions.insert(fn_id.clone(), Some(fn_tree));
            } else {
                remaining_functions.insert(fn_id.clone(), uninterpreted_fn.clone());
            }
        }

        // Now we iteratevely go through the remaining functions. If we find one referencing
        // only the already processed functions, we process its expression.
        // We made sure there are no cycles, so the process will terminate.
        while !remaining_functions.is_empty() {
            let mut done_in_this_iter = HashSet::new();
            for (unprocessed_fn_id, unprocessed_fn) in remaining_functions.iter() {
                let referenced_fn_symbols = unprocessed_fn.collect_fn_symbols();
                let not_ready_fn_symbols: HashSet<&UninterpretedFnId> = referenced_fn_symbols
                    .iter()
                    .filter(|fn_id| !processed_functions.contains_key(fn_id))
                    .collect();
                if !not_ready_fn_symbols.is_empty() {
                    continue;
                }
                let processed_tree = self.substitute_expressions_to_uninterpreted_fn(
                    unprocessed_fn,
                    &processed_functions,
                )?;
                processed_functions.insert(unprocessed_fn_id.clone(), Some(processed_tree));
                done_in_this_iter.insert((*unprocessed_fn_id).clone());
            }
            for done_fn in done_in_this_iter.iter() {
                remaining_functions.remove(done_fn);
            }
        }
        Ok(processed_functions)
    }

    /// Assert there are no cycles in the expressions of uninterpreted functions.
    /// A cycle is a situation where a function's expression references another
    /// function that, directly or indirectly, references the first function.
    ///
    /// If there is a cycle, this method will return an error with a message.
    pub fn assert_no_cycles_in_fn_expressions(&self) -> Result<(), String> {
        for (fn_id, uninterpreted_fn) in self.uninterpreted_fns() {
            // strart with the directly referenced functions
            let mut all_derived_fn_symbols = uninterpreted_fn.collect_fn_symbols();
            // recursively check if the function's expression contains a cycle
            loop {
                let mut new_derived_fns: HashSet<UninterpretedFnId> = HashSet::new();
                for derived_fn_id in &all_derived_fn_symbols {
                    // add all function symbols from the derived function's expression
                    if let Ok(derived_fn) = self.get_uninterpreted_fn(derived_fn_id) {
                        new_derived_fns.extend(derived_fn.collect_fn_symbols());
                    }
                }
                // if we found new derived functions, we need to check them too
                if !new_derived_fns.is_subset(&all_derived_fn_symbols) {
                    all_derived_fn_symbols.extend(new_derived_fns);
                } else {
                    break;
                }
            }

            if all_derived_fn_symbols.contains(fn_id) {
                return Err(format!(
                    "Recursion detected in the expression of function '{fn_id}'."
                ));
            }
        }
        Ok(())
    }

    /// Find all uninterpreted functions that are not used in any update function,
    /// even after propagating their expressions.
    ///
    /// There must be no cycles in the expressions of uninterpreted functions,
    /// otherwise this method will return error.
    ///
    /// For example, if we have PSBN with `A: f(B); B: A & B` and the following functions:
    /// - `f(x) = g(x) | x`
    /// - `g(x) = !x`
    /// - `h(x) = i(x) | x`
    /// - `i(x) = x`
    ///
    /// Then functions `h` and `i` are unused, as they are not used in any update function.
    pub fn find_redundant_uninterpreted_fns(&self) -> Result<HashSet<UninterpretedFnId>, String> {
        // Assert there are no cycle references (so that the algo terminates)
        self.assert_no_cycles_in_fn_expressions()?;

        // Slowly collect all used functions, and then subtract them from all functions
        let mut used_fn_symbols: HashSet<UninterpretedFnId> = HashSet::new();
        let all_fn_symbols: HashSet<UninterpretedFnId> =
            self.uninterpreted_fns.keys().cloned().collect();

        // Start by collecting all functions that are used in update functions directly
        for (_, update_fn) in self.update_fns() {
            used_fn_symbols = used_fn_symbols
                .union(&update_fn.collect_fn_symbols())
                .cloned()
                .collect();
        }

        // Then iteratively collect all functions that are present in expressions of
        // already collected used functions
        let mut added_functions_last_iter = used_fn_symbols.clone();
        while !added_functions_last_iter.is_empty() {
            let mut newly_added_functions: HashSet<UninterpretedFnId> = HashSet::new();
            for fn_id in &added_functions_last_iter {
                if let Ok(uninterpreted_fn) = self.get_uninterpreted_fn(fn_id) {
                    let fn_symbols_present = uninterpreted_fn.collect_fn_symbols();
                    newly_added_functions = newly_added_functions
                        .union(&fn_symbols_present)
                        .cloned()
                        .collect();
                }
            }
            // Add newly found functions to the used set
            used_fn_symbols = used_fn_symbols
                .union(&added_functions_last_iter)
                .cloned()
                .collect();
            added_functions_last_iter = newly_added_functions;
        }

        // Subtract used functions from all functions to get the unused ones
        let redundant_symbols = all_fn_symbols
            .difference(&used_fn_symbols)
            .cloned()
            .collect();
        Ok(redundant_symbols)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, vec};

    use crate::sketchbook::ids::UninterpretedFnId;
    use crate::sketchbook::model::{FnTree, ModelState};

    #[test]
    fn test_substitution_update_fn() {
        // Example simple PSBN with `A: f(A) | g(A)` and the following functions:
        // - `f(x) = !x`
        // - `g(x) = unspecified`

        let mut model = ModelState::new_with_vars(vec![("A", "A")]).unwrap();
        let var_a = model.get_var_id("A").unwrap();

        model
            .add_multiple_uninterpreted_fns(vec![("f", "f", 1), ("g", "g", 1)])
            .unwrap();
        let f_id = model.get_uninterpreted_fn_id("f").unwrap();
        let g_id = model.get_uninterpreted_fn_id("g").unwrap();
        model.set_update_fn(&var_a, "f(A) | g(A)").unwrap();
        model
            .set_uninterpreted_fn_expression(&f_id, "!var0")
            .unwrap();

        let f_fn = model.get_uninterpreted_fn(&f_id).unwrap();
        let expression_mapping = HashMap::from([
            (f_id.clone(), f_fn.get_fn_tree().clone()),
            (g_id.clone(), None),
        ]);

        let orig_update_fn = model.get_update_fn(&var_a).unwrap();
        let result_expression_tree = model
            .substitute_expressions_to_update_fn(orig_update_fn, &expression_mapping)
            .unwrap();
        let expected_expression_tree = FnTree::try_from_str("!A | g(A)", &model, None).unwrap();
        assert_eq!(result_expression_tree, expected_expression_tree);
    }

    #[test]
    fn test_substitution_uninterpreted_fn() {
        // Example with substituting `g(x) = unspecified` and `h(x) = !x` into
        // the expression `f(x) = g(x) | h(y)`.

        let mut model = ModelState::new_empty();
        model
            .add_multiple_uninterpreted_fns(vec![("f", "f", 1), ("g", "g", 1), ("h", "h", 1)])
            .unwrap();
        let f_id = model.get_uninterpreted_fn_id("f").unwrap();
        let g_id = model.get_uninterpreted_fn_id("g").unwrap();
        let h_id = model.get_uninterpreted_fn_id("h").unwrap();

        model
            .set_uninterpreted_fn_expression(&f_id, "g(var0) | h(var0)")
            .unwrap();
        model
            .set_uninterpreted_fn_expression(&h_id, "!var0")
            .unwrap();

        let h_fn = model.get_uninterpreted_fn(&h_id).unwrap();
        let expression_mapping = HashMap::from([
            (g_id.clone(), None),
            (h_id.clone(), h_fn.get_fn_tree().clone()),
        ]);

        let orig_f_fn = model.get_uninterpreted_fn(&f_id).unwrap();
        let result_expression_tree = model
            .substitute_expressions_to_uninterpreted_fn(orig_f_fn, &expression_mapping)
            .unwrap();
        let expected_expression_tree =
            FnTree::try_from_str("g(var0) | !var0", &model, Some(&f_id)).unwrap();
        assert_eq!(result_expression_tree, expected_expression_tree);
    }

    #[test]
    fn test_redundant_fn_detection() {
        // Example PSBN with `A: f(B); B: A & B` and the following functions:
        // - `f(x) = g(x) | x`
        // - `g(x) = !x`
        // - `h(x) = i(x) | x`
        // - `i(x) = x`

        let mut model = ModelState::new_with_vars(vec![("A", "A"), ("B", "B")]).unwrap();
        model
            .add_multiple_regulations(vec!["A -?? B", "B -?? A", "B -?? B"])
            .unwrap();
        model
            .add_multiple_uninterpreted_fns(vec![
                ("f", "f", 1),
                ("g", "g", 1),
                ("h", "h", 1),
                ("i", "i", 1),
            ])
            .unwrap();
        model
            .set_update_fn(&model.get_var_id("A").unwrap(), "f(B)")
            .unwrap();
        model
            .set_update_fn(&model.get_var_id("B").unwrap(), "A & B")
            .unwrap();
        model
            .set_uninterpreted_fn_expression_by_str("f", "g(var0) | var0")
            .unwrap();
        model
            .set_uninterpreted_fn_expression_by_str("g", "!var0")
            .unwrap();
        model
            .set_uninterpreted_fn_expression_by_str("h", "i(var0) | var0")
            .unwrap();

        // Functions `h` and `i` should be detected as unused
        let unused_fns = model.find_redundant_uninterpreted_fns().unwrap();
        assert_eq!(unused_fns.len(), 2);
        assert!(unused_fns.contains(&UninterpretedFnId::new("h").unwrap()));
        assert!(unused_fns.contains(&UninterpretedFnId::new("i").unwrap()));
    }

    #[test]
    fn test_cycle_detection_positive() {
        // Example model with the following functions with cycle in expressions:
        // - `f(x) = g(x) | x`
        // - `g(x) = !x & h(x)`
        // - `h(x) = f(x) | x`
        let mut model = ModelState::new_empty();
        model
            .add_multiple_uninterpreted_fns(vec![("f", "f", 1), ("g", "g", 1), ("h", "h", 1)])
            .unwrap();
        model
            .set_uninterpreted_fn_expression_by_str("f", "g(var0) | var0")
            .unwrap();
        model
            .set_uninterpreted_fn_expression_by_str("g", "!var0 & h(var0)")
            .unwrap();
        model
            .set_uninterpreted_fn_expression_by_str("h", "f(var0) | var0")
            .unwrap();

        assert!(model.assert_no_cycles_in_fn_expressions().is_err());
    }

    #[test]
    fn test_cycle_detection_negative() {
        // Example model with the following functions with no cycle in expressions:
        // - `f(x) = g(x) | x`
        // - `g(x) = !x & h(x)`
        // - `h(x) = x`
        let mut model = ModelState::new_empty();
        model
            .add_multiple_uninterpreted_fns(vec![("f", "f", 1), ("g", "g", 1), ("h", "h", 1)])
            .unwrap();
        model
            .set_uninterpreted_fn_expression_by_str("f", "g(var0) | var0")
            .unwrap();
        model
            .set_uninterpreted_fn_expression_by_str("g", "!var0 & h(var0)")
            .unwrap();
        model
            .set_uninterpreted_fn_expression_by_str("h", "var0")
            .unwrap();

        assert!(model.assert_no_cycles_in_fn_expressions().is_ok());
    }

    #[test]
    fn test_propagate_fn_expressions() {
        // Example using functions with the following expressions:
        // - `f(x, y) = g(x, x) | h(y)`
        // - `g(x, y) = !h(y) | i(x)`
        // - `h(x) = !x`
        // - `i(x) ... unspecified`
        let mut model = ModelState::new_empty();
        model
            .add_multiple_uninterpreted_fns(vec![
                ("f", "f", 2),
                ("g", "g", 2),
                ("h", "h", 1),
                ("i", "i", 1),
            ])
            .unwrap();
        let fn_f = model.get_uninterpreted_fn_id("f").unwrap();
        let fn_g = model.get_uninterpreted_fn_id("g").unwrap();
        let fn_h = model.get_uninterpreted_fn_id("h").unwrap();
        let fn_i = model.get_uninterpreted_fn_id("i").unwrap();

        model
            .set_uninterpreted_fn_expression(&fn_f, "g(var0, var0) | h(var1)")
            .unwrap();
        model
            .set_uninterpreted_fn_expression(&fn_g, "!h(var1) | i(var0)")
            .unwrap();
        model
            .set_uninterpreted_fn_expression(&fn_h, "!var0")
            .unwrap();

        let expected_f =
            FnTree::try_from_str("(!!var0 | i(var0)) | !var1", &model, Some(&fn_f)).unwrap();
        let expected_g = FnTree::try_from_str("!!var1 | i(var0)", &model, Some(&fn_g)).unwrap();
        let expected_h = FnTree::try_from_str("!var0", &model, Some(&fn_h)).unwrap();
        let expected_expressions_mapping = HashMap::from([
            (fn_f, Some(expected_f)),
            (fn_g, Some(expected_g)),
            (fn_h, Some(expected_h)),
            (fn_i, None),
        ]);
        let result_expressions_mapping = model
            .propagate_expressions_through_uninterpreted_fns()
            .unwrap();
        assert_eq!(result_expressions_mapping, expected_expressions_mapping);
    }
}
