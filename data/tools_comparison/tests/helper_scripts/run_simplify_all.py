import glob
import os
import sys
from sympy.logic.boolalg import simplify_logic
from sympy.parsing.sympy_parser import parse_expr


def simplify_expression(expr_str):
    """
    Parse any AEON boolean string, simplify it (e.g., `A | (!A & B)` -> `A | B`),
    and return the cleaned DNF expression. Simplification is done with `sympy`.
    """
    if not expr_str.strip():
        return expr_str

    # Convert AEON "not" syntax (!) to SymPy syntax (~)
    sympy_friendly = expr_str.strip().replace('!', '~')
    
    try:
        # Parse expression and convert to DNF
        expr = parse_expr(sympy_friendly)
        simplified = simplify_logic(expr, form='dnf', force=True)
        
        # Translate back to AEON format
        clean_str = str(simplified).replace('~', '!')
        return clean_str
    except Exception as e:
        print(f"    [!] Error parsing expression '{expr_str.strip()}': {e}")
        return expr_str # Return original if simplification fails

def process_file_inplace(filepath):
    """Process AEON file in place, simplifying the update expressions."""
    print(f"Processing: {os.path.basename(filepath)}...", end="")
    with open(filepath, 'r') as f:
        lines = f.readlines()
    
    modified_lines = []
    changes_count = 0
    for line in lines:
        # Update function lines start with $
        stripped = line.strip()
        if stripped.startswith('$') and ':' in stripped:
            var_name, expression = stripped.split(':', 1)
            
            # Simplify the expression and reconstruct the line
            new_expression = simplify_expression(expression)
            new_line = f"{var_name}: {new_expression}\n"
            
            # Check if we actually changed anything (ignoring whitespace)
            if new_line.strip().replace(" ", "") != line.strip().replace(" ", ""):
                changes_count += 1
            modified_lines.append(new_line)
        else:
            # Keep regulation lines and comments exactly as is
            modified_lines.append(line)
    
    # Write back to the same file (overwrite)
    with open(filepath, 'w') as f:
        f.writelines(modified_lines)
    print(f" Done. ({changes_count} lines optimized)")


def main():
    if len(sys.argv) < 2:
        print("Usage: python run_simplify_all.py <target_dir>")
        sys.exit(1)

    target_dir = sys.argv[1]
    if not os.path.isdir(target_dir):
        print(f"Error: Directory '{target_dir}' does not exist.")
        sys.exit(1)

    # Find all .aeon files
    search_path = os.path.join(target_dir, "*.aeon")
    aeon_files = glob.glob(search_path)
    if not aeon_files:
        print(f"No .aeon files found in {os.path.abspath(target_dir)}")
        return
    print(f"Found {len(aeon_files)} AEON files in '{target_dir}'. Overwriting in progress...\n")

    # Process each file, changing it in place
    for filepath in aeon_files:
        process_file_inplace(filepath)

    print("\nBatch processing complete.")


if __name__ == "__main__":
    main()