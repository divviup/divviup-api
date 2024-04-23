import AccountDetailFull from "./TaskList";
import TaskForm from "./TaskForm";
import TaskDetail from "./TaskDetail";
import ApiClient from "../ApiClient";
import { RouteObject, defer, redirect } from "react-router-dom";

export default function tasks(apiClient: ApiClient): RouteObject {
  return {
    path: "tasks",
    children: [
      {
        path: "",
        index: true,
        element: <AccountDetailFull />,
        loader({ params }) {
          return defer({
            tasks: apiClient.accountTasks(params.accountId as string),
          });
        },
      },
      {
        path: ":taskId",
        element: <TaskDetail />,
        async loader({ params }) {
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
              const data = Object.fromEntries(
                Array.from((await request.formData()).entries()).map(
                  (entry): [string, string | null] => [
                    entry[0],
                    entry[1].toString(),
                  ],
                ),
              );
              const expiration = data["expiration"];
              data["expiration"] = expiration === "" ? null : expiration;

              return apiClient.updateTask(
                params.taskId as string,
                data as { name: string } | { expiration: string | null },
              );
            }
            case "DELETE": {
              await apiClient.deleteTask(params.taskId as string);
              return redirect("..");
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
