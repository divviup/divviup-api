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
            tasks: apiClient.accountTasks(params.accountId as string),
          });
        },
      },
      {
        path: ":taskId",
        element: <TaskDetail />,
        loader({ params }) {
          const task = apiClient.task(params.taskId as string);
          const leaderAggregator = task.then((t) =>
            apiClient.aggregator(t.leader_aggregator_id),
          );
          const helperAggregator = task.then((t) =>
            apiClient.aggregator(t.helper_aggregator_id),
          );
          const collectorCredential = task.then((t) =>
            apiClient.collectorCredential(t.collector_credential_id),
          );
          return defer({
            task,
            leaderAggregator,
            helperAggregator,
            collectorCredential,
          });
        },

        async action({ params, request }) {
          switch (request.method) {
            case "PATCH": {
              return apiClient.updateTask(
                params.taskId as string,
                (await request.json()) as { name: string },
              );
            }
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
                  params.taskId as string,
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
              params.accountId as string,
            ),
            collectorCredentials: apiClient.accountCollectorCredentials(
              params.accountId as string,
            ),
          });
        },
      },
    ],
  };
}
