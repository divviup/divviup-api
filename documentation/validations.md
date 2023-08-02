# Validations

## Tasks

* name
  * required
  * must be at least one character
* leader aggregator
  * required
  * cannot be the same as the helper aggregator
  * must be "first party" if the helper aggregator is not
  * must be willing to be leader
* helper aggregator
  * required
  * cannot be the same as the leader aggregator
  * must be "first party" if the leader aggregator is not
  * must be willing to be helper
* vdaf (function)
  * required
  * must be supported by the leader and the helper
  * histogram buckets
    * must be a strictly-increasing list of positive integers
  * sum bits
    * must be a number (0-255)
* min batch size
  * required
  * minimum: 100
* max batch size (query type)
  * must be a positive integer if present, absence means time interval query type
  * query type must be supported by both aggregators
* expiration
  * must be in the future if present
* time precision
  * required
  * must be between one minute and four weeks
* hpke config
  * required
  * must be correctly formatted (standard base64 of DAP-encoding)


## Aggregators

* name
  * required
  * must be at least one character
* api url
  * required
  * must have a `https` url scheme
* bearer token
  * not actually validated, but checked against the aggregator

