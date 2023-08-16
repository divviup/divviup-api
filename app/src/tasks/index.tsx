import React from "react";
import TaskList from "./TaskList";
import TaskForm from "./TaskForm";
import TaskDetail from "./TaskDetail";
import ApiClient from "../ApiClient";
import { RouteObject, defer } from "react-router-dom";

export default function tasks(apiClient: ApiClient): RouteObject {
  return {
    path: "tasks",
    children: [
      {
        path: "",
        index: true,
        element: <TaskList />,
        loader({ params }) {
          return defer({
            tasks: apiClient.accountTasks(params.account_id as string),
          });
        },
      },
      {
        path: ":task_id",
        element: <TaskDetail />,
        loader({ params }) {
          const task = apiClient.task(params.task_id as string);
          const leaderAggregator = task.then((t) =>
            apiClient.aggregator(t.leader_aggregator_id),
          );
          const helperAggregator = task.then((t) =>
            apiClient.aggregator(t.helper_aggregator_id),
          );
          const hpkeConfig = task.then((t) =>
            apiClient.hpkeConfig(t.hpke_config_id),
          );
          return defer({
            task,
            leaderAggregator,
            helperAggregator,
            hpkeConfig,
          });
        },

        async action({ params, request }) {
          const data = Object.fromEntries(await request.formData());
          switch (request.method) {
            case "PATCH":
              return await apiClient.updateTask(
                params.task_id as string,
                data as { name: string },
              );
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },
        children: [
          {
            path: "collector_auth_tokens",
            loader({ params }) {
              return defer({
                collectorAuthTokens: apiClient.taskCollectorAuthTokens(
                  params.task_id as string,
                ),
              });
            },
          },
        ],
      },
      {
        path: "new",
        element: <TaskForm />,
        loader({ params }) {
          return defer({
            aggregators: apiClient.accountAggregators(
              params.account_id as string,
            ),
            hpkeConfigs: apiClient.accountHpkeConfigs(
              params.account_id as string,
            ),
          });
        },
      },
    ],
  };
}
