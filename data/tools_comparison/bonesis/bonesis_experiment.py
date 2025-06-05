import bonesis as bo
import bonesis.aeon
import biodivine_aeon as aeon


# load the partially specified model
bn = aeon.BooleanNetwork.from_file("arabidopsis_sketch.aeon")
dom = bonesis.aeon.AEONDomain(bn)

# prepare the data
data = {
    'f1': { "AGO1": 1, "AGO10": 0, "AGO7": 0, "ANT": 1, "ARF4": 1, "AS1": 0, "AS2": 0, "ETT": 1, "FIL": 1, "KAN1": 1, "miR165": 1, "miR390": 1, "REV": 0, "TAS3siRNA": 0, "AGO1": 1, "_miR165": 1, "AGO7": 0, "_miR390": 1, "AS1": 0, "_AS2": 1, "AUXINh": 1, "CKh": 0, "GTE6": 0, "IPT5": 0, },
    'f2': {"AGO1": 0, "AGO10": 1, "AGO7": 1, "ANT": 1, "ARF4": 0, "AS1": 1, "AS2": 1, "ETT": 0, "FIL": 0, "KAN1": 0, "miR165": 0, "miR390": 1, "REV": 1, "TAS3siRNA": 1, "AGO1": 0, "_miR165": 1, "AGO7": 1, "_miR390": 1, "AS1": 1, "_AS2": 1, "AUXINh": 1, "CKh": 1, "GTE6": 1, "IPT5": 1, }
}

# create the solver and set it up
bo = bonesis.BoNesis(dom, data)
bo.fixed(~bo.obs('f1'))
bo.fixed(~bo.obs('f2'))
#bo.all_fixpoints({bo.obs(obs) for obs in ["f1", "f2"]});

# run the inference, counting the results
count_candidates = bo.boolean_networks(limit=1000000).count()
print(f"There are {count_candidates} candidate networks.")
