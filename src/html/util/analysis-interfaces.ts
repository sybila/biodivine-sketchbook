/** Enum with different variants of inference. */
export enum InferenceType {
  FullInference = 'FullInference',
  StaticInference = 'StaticInference',
  DynamicInference = 'DynamicInference'
}

/** Typesafe representation of statuses of the inference computation. */
export type InferenceStatus =
    | 'Started'
    | 'ProcessedInputs'
    | 'GeneratedContextStatic'
    | 'GeneratedContextDynamic'
    | { EvaluatedStatic: string } // EvaluatedStatic(String)
    | 'EvaluatedAllStatic'
    | { EvaluatedDynamic: string } // EvaluatedDynamic(String)
    | 'EvaluatedAllDynamic'
    | 'DetectedUnsat'
    | 'FinishedSuccessfully'

/** Report with a summary of the inference computaiton. */
export interface InferenceStatusReport {
  status: InferenceStatus
  // This value is represented as string, since it can exceed the maximum number size in JavaScript
  num_candidates: string | null
  comp_time: number
  message: string
}

/** Structure representing all information regarding inference results. */
export interface InferenceResults {
  analysis_type: InferenceType
  num_sat_networks: number
  comp_time: number
  summary_message: string
  progress_statuses: InferenceStatusReport[]
  num_update_fns_per_var: Record<string, number>
}
