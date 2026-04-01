// Original file: scraper.proto

import type { Long } from '@grpc/proto-loader';

export interface ItemFilter {
  'job_id'?: (string);
  'status'?: (string);
  'offset'?: (number | string | Long);
  'limit'?: (number | string | Long);
}

export interface ItemFilter__Output {
  'job_id': (string);
  'status': (string);
  'offset': (string);
  'limit': (string);
}
