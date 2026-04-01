// Original file: scraper.proto


export interface Source {
  'id'?: (string);
  'created'?: (string);
  'updated'?: (string);
  'name'?: (string);
  'source_url'?: (string);
  'source_type'?: (string);
  'integration_mode'?: (string);
  'auto_schedule'?: (string);
  'field_rules'?: (string);
  'active'?: (boolean);
}

export interface Source__Output {
  'id': (string);
  'created': (string);
  'updated': (string);
  'name': (string);
  'source_url': (string);
  'source_type': (string);
  'integration_mode': (string);
  'auto_schedule': (string);
  'field_rules': (string);
  'active': (boolean);
}
