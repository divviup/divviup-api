import { RouteObject } from "react-router";
import ApiClient from "../ApiClient";

export default function queue(apiClient: ApiClient): RouteObject {
  return {
    path: "queue",
    async lazy() {
      return import("./Queue");
    },
    async loader({ request }) {
      const params = new URL(request.url).searchParams;
      return apiClient.queue(params);
    },

    children: [
      {
        path: ":job_id",
        async lazy() {
          return import("./QueueJob");
        },

        async loader({ params }) {
          if ("job_id" in params && typeof params.job_id === "string")
            return apiClient.queueJob(params.job_id);
        },
      },
    ],
  };
}
