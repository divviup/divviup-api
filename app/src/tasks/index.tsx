import AccountDetailFull from "./TaskList";
import TaskForm from "./TaskForm";
import TaskDetail from "./TaskDetail";
import ApiClient from "../ApiClient";
import { RouteObject, redirect } from "react-router";

export default function tasks(apiClient: ApiClient): RouteObject {
  return {
    path: "tasks",
    children: [
      {
        path: "",
        index: true,
        element: <AccountDetailFull />,
        loader({ params }) {
          return {
            tasks: apiClient.accountTasks(params.accountId as string),
          };
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
          return {
            task,
            leaderAggregator,
            helperAggregator,
            collectorCredential,
          };
        },

        async action({ params, request }) {
          switch (request.method) {
            case "PATCH": {
              if (request.headers.get("Content-Type") === "application/json") {
                return apiClient.updateTask(
                  params.taskId as string,
                  await request.json(),
                );
              } else {
                const data = Object.fromEntries(await request.formData());
                return apiClient.updateTask(
                  params.taskId as string,
                  data as { name: string },
                );
              }
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
              return {
                collectorAuthTokens: apiClient.taskCollectorAuthTokens(
                  params.taskId as string,
                ),
              };
            },
          },
        ],
      },
      {
        path: "new",
        element: <TaskForm />,
        loader({ params }) {
          return {
            aggregators: apiClient.accountAggregators(
              params.accountId as string,
            ),
            collectorCredentials: apiClient.accountCollectorCredentials(
              params.accountId as string,
            ),
          };
        },
      },
    ],
  };
}
