export enum InferenceType {
  FullInference = 'FullInference',
  StaticInference = 'StaticInference',
  DynamicInference = 'DynamicInference'
}

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

export interface InferenceStatusReport {
  status: InferenceStatus
  num_candidates: number | null
  comp_time: number
  message: string
}

/** An object representing all information regarding inference analysis results. */
export interface InferenceResults {
  analysis_type: InferenceType
  num_sat_networks: number
  comp_time: number
  summary_message: string
  progress_statuses: InferenceStatusReport[]
}
