// Original file: scraper.proto


export interface Item {
  'id'?: (string);
  'created'?: (string);
  'updated'?: (string);
  'job_id'?: (string);
  'raw_data'?: (string);
  'validation_status'?: (string);
  'validation_errors'?: (string);
  'pushed_at'?: (string);
  'pushed_target'?: (string);
}

export interface Item__Output {
  'id': (string);
  'created': (string);
  'updated': (string);
  'job_id': (string);
  'raw_data': (string);
  'validation_status': (string);
  'validation_errors': (string);
  'pushed_at': (string);
  'pushed_target': (string);
}
