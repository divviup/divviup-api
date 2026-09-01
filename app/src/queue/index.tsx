import { RouteObject } from "react-router";
import ApiClient from "../ApiClient.js";

export default function queue(apiClient: ApiClient): RouteObject {
  return {
    path: "queue",
    async lazy() {
      return import("./Queue.js");
    },
    async loader({ request }) {
      const params = new URL(request.url).searchParams;
      return apiClient.queue(params);
    },

    children: [
      {
        path: ":jobId",
        async lazy() {
          return import("./QueueJob.js");
        },

        async loader({ params }) {
          if ("jobId" in params && typeof params.jobId === "string")
            return apiClient.queueJob(params.jobId);
        },
      },
    ],
  };
}
