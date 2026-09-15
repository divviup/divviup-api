import { render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router";
import TaskDetail from "./TaskDetail/index.js";
import {
  Account,
  Aggregator,
  CollectorCredential,
  Task,
} from "../ApiClient.js";

test("TaskDetail renders", async () => {
  const router = createMemoryRouter(
    [
      {
        path: "/accounts/:account_id",
        id: "account",
        async loader({ params }) {
          const { accountId } = params as { accountId: string };
          return {
            account: (async (): Promise<Account> => {
              return {
                name: "Test account",
                id: accountId,
                created_at: "1985-04-12T23:20:50.52Z",
                updated_at: "1985-04-12T23:20:50.52Z",
                intends_to_use_shared_aggregators: false,
                admin: false,
              };
            })(),
          };
        },
        children: [
          {
            path: "tasks/:taskId",
            element: <TaskDetail />,
            async loader({ params }) {
              const { accountId, taskId } = params as {
                accountId: string;
                taskId: string;
              };
              return {
                task: (async (): Promise<Task> => {
                  return {
                    id: taskId,
                    name: "Counter task",
                    leader_aggregator_id:
                      "00000000-0000-0000-0000-00000000000a",
                    helper_aggregator_id:
                      "00000000-0000-0000-0000-00000000000b",
                    vdaf: { type: "count" },
                    min_batch_size: 100,
                    time_precision_seconds: 3600,
                    account_id: accountId,
                    created_at: "1985-04-12T23:20:50.52Z",
                    updated_at: "1985-04-12T23:20:50.52Z",
                    expiration: null,
                    max_batch_size: null,
                    batch_time_window_size_seconds: null,
                    collector_credential_id:
                      "00000000-0000-0000-0000-00000000000c",
                    report_counter_interval_collected: 0,
                    report_counter_decode_failure: 0,
                    report_counter_decrypt_failure: 0,
                    report_counter_expired: 0,
                    report_counter_outdated_key: 0,
                    report_counter_success: 0,
                    report_counter_too_early: 0,
                    report_counter_task_expired: 0,
                    aggregation_job_counter_success: 0,
                    aggregation_job_counter_helper_batch_collected: 0,
                    aggregation_job_counter_helper_report_replayed: 0,
                    aggregation_job_counter_helper_report_dropped: 0,
                    aggregation_job_counter_helper_hpke_unknown_config_id: 0,
                    aggregation_job_counter_helper_hpke_decrypt_failure: 0,
                    aggregation_job_counter_helper_vdaf_prep_error: 0,
                    aggregation_job_counter_helper_task_expired: 0,
                    aggregation_job_counter_helper_invalid_message: 0,
                    aggregation_job_counter_helper_report_too_early: 0,
                  };
                })(),
                leaderAggregator: (async (): Promise<Aggregator> => {
                  return {
                    id: "00000000-0000-0000-0000-00000000000a",
                    account_id: accountId,
                    created_at: "1985-04-12T23:20:50.52Z",
                    updated_at: "1985-04-12T23:20:50.52Z",
                    deleted_at: null,
                    api_url: "http://leader.invalid/aggregator-api/",
                    dap_url: "http://leader.invalid/",
                    role: "Leader",
                    name: "Leader aggregator",
                    is_first_party: true,
                    vdafs: [
                      "Prio3Count",
                      "Prio3Sum",
                      "Prio3Histogram",
                      "Prio3SumVec",
                    ],
                    query_types: ["TimeInterval", "FixedSize"],
                    features: [
                      "TokenHash",
                      "UploadMetrics",
                      "TimeBucketedFixedSize",
                      "PureDpDiscreteLaplace",
                      "AggregationJobMetrics",
                    ],
                    protocol: "DAP-09",
                  };
                })(),
                helperAggregator: (async (): Promise<Aggregator> => {
                  return {
                    id: "00000000-0000-0000-0000-00000000000b",
                    account_id: accountId,
                    created_at: "1985-04-12T23:20:50.52Z",
                    updated_at: "1985-04-12T23:20:50.52Z",
                    deleted_at: null,
                    api_url: "http://helper.invalid/aggregator-api/",
                    dap_url: "http://helper.invalid/",
                    role: "Helper",
                    name: "Helper aggregator",
                    is_first_party: false,
                    vdafs: [
                      "Prio3Count",
                      "Prio3Sum",
                      "Prio3Histogram",
                      "Prio3SumVec",
                    ],
                    query_types: ["TimeInterval", "FixedSize"],
                    features: [
                      "TokenHash",
                      "UploadMetrics",
                      "TimeBucketedFixedSize",
                      "PureDpDiscreteLaplace",
                      "AggregationJobMetrics",
                    ],
                    protocol: "DAP-09",
                  };
                })(),
                collectorCredential:
                  (async (): Promise<CollectorCredential> => {
                    return {
                      id: "00000000-0000-0000-0000-00000000000c",
                      hpke_config: {
                        id: 0,
                        kem_id: "X25519HkdfSha256",
                        kdf_id: "HkdfSha256",
                        aead_id: "Aes128Gcm",
                        public_key:
                          "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                      },
                      created_at: "1985-04-12T23:20:50.52Z",
                      updated_at: "1985-04-12T23:20:50.52Z",
                      deleted_at: null,
                      name: "Test collector credential",
                      token_hash: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                    };
                  })(),
              };
            },
          },
        ],
      },
    ],
    {
      initialEntries: [
        "/accounts/00000000-0000-0000-0000-000000000000/tasks/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
      ],
    },
  );

  render(<RouterProvider router={router} />);

  await screen.findByText("Minimum Batch Size: 100");
});
