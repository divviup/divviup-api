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
          let task = apiClient.task(params.task_id as string);
          let leaderAggregator = task.then((t) =>
            apiClient.aggregator(t.leader_aggregator_id)
          );
          let helperAggregator = task.then((t) =>
            apiClient.aggregator(t.helper_aggregator_id)
          );
          let hpkeConfig = task.then((t) =>
            apiClient.hpkeConfig(t.hpke_config_id)
          );
          return defer({
            task,
            leaderAggregator,
            helperAggregator,
            hpkeConfig,
          });
        },

        async action({ params, request }) {
          let data = Object.fromEntries(await request.formData());
          switch (request.method) {
            case "PATCH":
              return await apiClient.updateTask(
                params.task_id as string,
                data as { name: string }
              );
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },
      },
      {
        path: "new",
        element: <TaskForm />,
        loader({ params }) {
          return defer({
            aggregators: apiClient.accountAggregators(
              params.account_id as string
            ),
            hpkeConfigs: apiClient.accountHpkeConfigs(
              params.account_id as string
            ),
          });
        },
      },
    ],
  };
}
