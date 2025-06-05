$FirstObs := {
 AGO1 = 1 and
 AGGGG = 0 and
 AGO7 = 0 and
 ANT = 1 and
 ARF4 = 1 and
 AS1 = 0 and
 AS2 = 0 and
 ETT = 1 and
 FIL = 1 and
 KAN1 = 1 and
 miR165 = 1 and
 miR390 = 1 and
 REV = 0 and
 KKKtomiR1 = 1 and
 KKKtomiR3 = 0 and
 TAS3siRNA = 0 and
 KKKAStoAS = 0 and
 AUXINh = 1 and
 CKh = 0 and
 GTE6 = 0 and
 IPT5 = 0
};

$SecondObs := {
 AGO1 = 0 and
 AGGGG = 1 and
 AGO7 = 1 and
 ANT = 1 and
 ARF4 = 0 and
 AS1 = 1 and
 AS2 = 1 and
 ETT = 0 and
 FIL = 0 and
 KAN1 = 0 and
 miR165 = 0 and
 miR390 = 1 and
 REV = 1 and
 KKKtomiR1 = 0 and
 KKKtomiR3 = 1 and
 TAS3siRNA = 1 and
 KKKAStoAS = 1 and
 AUXINh = 1 and
 CKh = 1 and
 GTE6 = 1 and
 IPT5 = 1
};

#AttractorOne[0] |= $FirstObs; 
fixpoint(#AttractorOne[0]);

#AttractorTwo[0] |= $SecondObs; 
fixpoint(#AttractorTwo[0]);
