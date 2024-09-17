A real case studies collected and adapted to BN sketches framework:

1) Apoptosis case study:
    - background: This model and expected attractor data is taken from [this paper](https://doi.org/10.3389/fgene.2018.00039) about the BN inference tool Griffin. Originally, the model was developed by La Rota et al. in [this paper](https://doi.org/10.1105/tpc.111.092619).
    - model: We use the fully unspecified model directly as is.
    - properties: We use two fixed-point properties adapted from the Griffin paper.
    - candidates after static: 4761711360
    - candidates after all: 439296

2) TLGL case study:
    - background: The original model was developed by Saadatpour et al. in [this paper](https://doi.org/10.1371/journal.pcbi.1002267). It is a reduced version of the model developed by Zhang et al. in [this article](https://doi.org/10.1073/pnas.0806447105). The experimental attractor data comes [this work](https://doi.org/10.1371/journal.pcbi.1002267) as well.
    - model: We use a partially specified version of the reduced model, only assuming that Apoptosis must inactivate all variables.
    - properties: We use a fixed-point property for a "healthy attractor" (cell death) and a complex attractor property for a "diseased attractor", based on provided data regarding the T-LGL state.
    - candidates after static: 1296
    - candidates after all: 486