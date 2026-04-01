// Original file: scraper.proto

import type { Job as _proto_Job, Job__Output as _proto_Job__Output } from '../proto/Job';

export interface JobResponse {
  'job'?: (_proto_Job | null);
  'source_name'?: (string);
}

export interface JobResponse__Output {
  'job': (_proto_Job__Output | null);
  'source_name': (string);
}
