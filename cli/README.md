## divviup - the Divvi Up command line tool

`divviup` is a command line (CLI) tool for doing all basic operations on both the Divvi Up API and Distributed Aggregation Protocol (DAP) API endpoints. It's only likely to work if the leader aggregator is [Janus](https://github.com/divviup/janus). See `divviup --help` for details on all of the commands.

### Command Line Tutorial

First, set an environment variable for the Divvi Up account and an API token. You can create an API token by logging into the [Divvi Up console](https://app.divviup.org).

```sh
export DIVVIUP_TOKEN=YOUR_TOKEN_HERE
```

Test the token by listing all accounts. Your account ID is inferred from the token.

```sh
divviup account list
```

List the aggregators and identify the ID of both a leader and a helper.

```sh
divviup aggregator list
```

The output will contain JSON objects like:

```json
  {
    "id": "3650870b-56e6-4eac-8944-b7ca36569b33",
    "role": "Either",
    "name": "Divvi Up staging-dap-09-1",
    "dap_url": "https://staging-dap-09-1.api.example.com/",
    "api_url": "https://staging-dap-09-1.api.example.com/aggregator-api",
    "is_first_party": true,
    "vdafs": [
      "Prio3Count",
      "Prio3Sum",
      "Prio3Histogram",
      "Prio3SumVec"
    ],
    "query_types": [
      "TimeInterval",
      "FixedSize"
    ],
    "protocol": "DAP-09",
    "features": [
      "TokenHash",
      "TimeBucketedFixedSize",
      "UploadMetrics"
    ]
  },
  {
    "id": "96301951-c848-4a57-b4f5-32812e4db1be",
    "account_id": null,
    "created_at": "2024-03-21T22:47:15.467139Z",
    "updated_at": "2024-04-18T17:33:30.439465Z",
    "deleted_at": null,
    "role": "Either",
    "name": "Divvi Up staging-dap-09-2",
    "dap_url": "https://staging-dap-09-2.api.example.com/",
    "api_url": "https://staging-dap-09-2.api.example.com/aggregator-api",
    "is_first_party": false,
    "vdafs": [
      "Prio3Count",
      "Prio3Sum",
      "Prio3Histogram",
      "Prio3SumVec"
    ],
    "query_types": [
      "TimeInterval",
      "FixedSize"
    ],
    "protocol": "DAP-09",
    "features": [
      "TokenHash",
      "UploadMetrics",
      "TimeBucketedFixedSize"
    ]
  }
```

Set the two IDs into environment variables. NOTE: These IDs will vary based on your configuration.

```sh
export LEADER_ID=3650870b-56e6-4eac-8944-b7ca36569b33
export HELPER_ID=96301951-c848-4a57-b4f5-32812e4db1be
```

Next, generate a collector-credential for the task. The collector credential will be used by the collector to export the aggregated statistics.

```sh
divviup collector-credential generate --save
```

The output will be like:

```sh
{
  "id": "0a0f8ea8-b603-4416-b138-b7f217153bb7",
  "hpke_config": {
    "id": 144,
    "kem_id": "X25519HkdfSha256",
    "kdf_id": "HkdfSha256",
    "aead_id": "Aes128Gcm",
    "public_key": "V9IpdJxS91MHPiNTjwDk9DFS-5M_neVrPxlmvolmTTo"
  },
  "created_at": "2024-05-03T15:23:56.624726Z",
  "deleted_at": null,
  "updated_at": "2024-05-03T15:23:56.624727Z",
  "name": "collector-credential-144",
  "token_hash": "VItYJdAyWYIvooe8GzkGnVTvaMkvWc9G-eiwxudfWww",
  "token": "A6JDAYPiYDXmNyh-OpYXGw"
}

Saved new collector credential to /your/current/directory/collector-credential-<some-number>.json. Keep this file safe!
```

Make a note of the path where the credential was saved, and save the collector credential ID to an environment variable.

```sh
export COLLECTOR_CREDENTIAL_PATH=/your/current/directory/collector-credential-<some-number>.json
export COLLECTOR_ID=0a0f8ea8-b603-4416-b138-b7f217153bb7
```

Create the the histogram task. In this case the task is a set of values from 0 to 10 for use in collecting a net-promoter score for a survey.

```sh
 divviup task create --name net-promoter-score \
    --leader-aggregator-id $LEADER_ID --helper-aggregator-id $HELPER_ID \
    --collector-credential-id $COLLECTOR_ID \
    --vdaf histogram --categorical-buckets 0,1,2,3,4,5,6,7,8,9,10 \
    --min-batch-size 100 --max-batch-size 200 --time-precision 60sec
```

The output will contain a JSON object:

```json
{
  "id": "Siwa4QTEnQXMfRPyhir8AzS4EBqfTebmEzKfvajDgYk",
  "account_id": "a9c571ba-5f3d-4814-8d8b-c5bb0f5030b7",
  "name": "net-promoter-score",
  "vdaf": {
    "type": "histogram",
    "buckets": [
      "0",
      "1",
      "2",
      "3",
      "4",
      "5",
      "6",
      "7",
      "8",
      "9",
      "10"
    ],
    "chunk_length": 4
  },
  "min_batch_size": 100,
  "max_batch_size": null,
  "created_at": "2024-05-03T15:27:56.229891Z",
  "updated_at": "2024-05-03T15:27:56.229891Z",
  "time_precision_seconds": 60,
  "report_count": 0,
  "aggregate_collection_count": 0,
  "expiration": "2025-05-03T15:27:55.88511Z",
  "leader_aggregator_id": "3650870b-56e6-4eac-8944-b7ca36569b33",
  "helper_aggregator_id": "96301951-c848-4a57-b4f5-32812e4db1be",
  "collector_credential_id": "0a0f8ea8-b603-4416-b138-b7f217153bb7",
  "report_counter_interval_collected": 0,
  "report_counter_decode_failure": 0,
  "report_counter_decrypt_failure": 0,
  "report_counter_expired": 0,
  "report_counter_outdated_key": 0,
  "report_counter_success": 0,
  "report_counter_too_early": 0,
  "report_counter_task_expired": 0
}
```

Save the ID of the task into an environment variable.

```sh
export TASK_ID=Siwa4QTEnQXMfRPyhir8AzS4EBqfTebmEzKfvajDgYk
```

Upload a random set of 150 metrics.

```sh
for i in {1..150}; do
  measurement=$(( $RANDOM % 10 ))
  divviup dap-client upload --task-id $TASK_ID  --measurement $measurement;
done
```

Wait a little while to let the aggregators run the aggregation jobs. Then, get collection results:

```sh
divviup dap-client collect \
    --task-id $TASK_ID \
    --collector-credential-file $COLLECTOR_CREDENTIAL_PATH \
    --current-batch
```

You will get a result like:

```sh
Number of reports: 113
Interval start: 2024-06-05 21:31:00 UTC
Interval end: 2024-06-05 21:34:00 UTC
Interval length: 180s
Aggregation result: [14, 10, 10, 13, 13, 8, 16, 13, 9, 7, 0]
collection: Collection { partial_batch_selector: PartialBatchSelector { batch_identifier: BatchId(wPLBlC6iHWp_YBBAYP_ig5nal0FOz1QlLSaC42U7sm0) }, report_count: 113, interval: (2024-06-05T21:31:00Z, TimeDelta { secs: 180, nanos: 0 }), aggregate_result: [14, 10, 10, 13, 13, 8, 16, 13, 9, 7, 0] }
```

If you get fewer reports in the collection than you uploaded, that's because you sent the collection request too soon and not all the reports were prepared yet. Those reports will be available in a later collection, after enough additional reports are uploaded to meet the minimum batch size.
