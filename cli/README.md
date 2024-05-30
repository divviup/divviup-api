## divviup - the Divvi Up command line tool

`divviup` is a command line (CLI) tool for doing all basic operations on both the Divvi Up API and Distributed Aggregation Protocol (DAP) API endpoints. It's only likely to work if the leader aggregator is [Janus](https://github.com/divviup/janus). See `divviup --help` for details on all of the commands.

### Command Line Tutorial

First, set an environment variable for the Divvi Up account and an API token. The initial token will be created when an account is created.

```
export DIVVIUP_TOKEN=YOUR_TOKEN_HERE
```

Test the token by listing all accounts.

```
divviup account list
```

List the aggregators and identify the ID of both a leader and a helper.

```
divviup aggregator list
```

```
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

```
export LEADERID=3650870b-56e6-4eac-8944-b7ca36569b33
export HELPERID=96301951-c848-4a57-b4f5-32812e4db1be
```

Next, generate a collector-credential for the task. The collector credential will be used by the collector to export the aggregated statistics.

```
divviup collector-credential generate --save
```

```
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

New collector credential generated. Copy and paste the following text into a file or your password manager:
{
  "aead": "AesGcm128",
  "id": 144,
  "kdf": "Sha256",
  "kem": "X25519HkdfSha256",
  "private_key": "SECRETS",
  "public_key": "V9IpdJxS91MHPiNTjwDk9DFS-5M_neVrPxlmvolmTTo",
  "token": "A6JDAYPiYDXmNyh-OpYXGw"
}
```

Save the collector credential ID to an environment variable.

```
export COLLECTORID=0a0f8ea8-b603-4416-b138-b7f217153bb7
```

Create the the histogram task. In this case the task is a set of values from 0 to 10 for use in collecting an net-promoter score for a survey.

```
 divviup task create --name net-promoter-score \
 --leader-aggregator-id $LEADERID --helper-aggregator-id $HELPERID \
 --collector-credential-id $COLLECTORID \
 --vdaf histogram --categorical-buckets 0,1,2,3,4,5,6,7,8,9,10 \
 --min-batch-size 100 --max-batch-size 200 --time-precision 60sec
```

```
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

```
export TASKID=Siwa4QTEnQXMfRPyhir8AzS4EBqfTebmEzKfvajDgYk
```

Upload a random set of 150 metrics.

```
for i in {1..150}; do
  value=$(( $RANDOM % 10 ))
  divviup dap-client upload --task-id $TASKID  --value $value;
done
```

TODO(timg) fill in guide for collecting aggregate result

```

```


```
./target/debug/collect  --task-id rc0jgm1MHH6Q7fcI4ZdNUxas9DAYLcJFK5CL7xUl-gU --leader https://staging-dap-09-2.api.divviup.org/ --length 12  --vdaf histogram  --collector-credential-file ~/collector.json  --current-batch
```

