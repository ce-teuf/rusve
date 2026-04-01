// Original file: scraper.proto

import type { Long } from '@grpc/proto-loader';

export interface Job {
  'id'?: (string);
  'created'?: (string);
  'updated'?: (string);
  'source_id'?: (string);
  'source_url'?: (string);
  'source_type'?: (string);
  'status'?: (string);
  'item_count'?: (number | string | Long);
  'error'?: (string);
}

export interface Job__Output {
  'id': (string);
  'created': (string);
  'updated': (string);
  'source_id': (string);
  'source_url': (string);
  'source_type': (string);
  'status': (string);
  'item_count': (string);
  'error': (string);
}
