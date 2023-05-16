import React from "react";
import { createBrowserRouter, RouterProvider, defer } from "react-router-dom";
import { ApiClientContext } from "./ApiClientContext";
import { ApiClient, PartialAccount } from "./ApiClient";
import AccountForm from "./AccountForm";
import AccountList from "./AccountList";
import Layout from "./Layout";
import Root from "./Root";
import TaskList from "./TaskList";
import Memberships from "./Memberships";
import AccountSummary from "./AccountSummary";
import TaskForm from "./TaskForm";
import TaskDetail from "./TaskDetail";
import { AxiosError } from "axios";
import ErrorPage from "./ErrorPage";

function Login({ apiClient }: { apiClient: ApiClient }) {
  apiClient.loginUrl().then((url) => {
    window.location.href = url;
  });
  return <></>;
}

function Logout({ apiClient }: { apiClient: ApiClient }) {
  apiClient.logoutUrl().then((url) => {
    window.location.href = url;
  });
  return <></>;
}

function buildRouter(apiClient: ApiClient) {
  return createBrowserRouter([
    {
      path: "/",
      element: <Layout />,
      id: "currentUser",
      loader: async () => ({
        currentUser: await apiClient.getCurrentUser().catch((e: unknown) => {
          if (e instanceof AxiosError) {
            if (e.response?.status === 403) return null;
          }
        }),
      }),

      errorElement: <ErrorPage apiClient={apiClient} />,

      children: [
        {
          path: "",
          element: <Root />,
          index: true,
        },
        {
          path: "admin",
          children: [
            {
              path: "queue",
              async lazy() {
                return import("./admin/Queue");
              },
              async loader({ request }) {
                const params = new URL(request.url).searchParams;
                return apiClient.queue(params);
              },
              children: [
                {
                  path: ":job_id",
                  async lazy() {
                    return import("./admin/QueueJob");
                  },

                  async loader({ params }) {
                    if ("job_id" in params && typeof params.job_id === "string")
                      return apiClient.queueJob(params.job_id);
                  },
                },
              ],
            },
          ],
        },

        {
          path: "login",
          element: <Login apiClient={apiClient} />,
        },
        { path: "logout", element: <Logout apiClient={apiClient} /> },
        {
          path: "accounts",

          children: [
            {
              path: "",
              element: <AccountList />,
              loader() {
                return defer({ accounts: apiClient.accounts() });
              },
              index: true,
            },
            {
              path: ":account_id",
              id: "account",
              loader({ params }) {
                return defer({
                  account: apiClient.account(params.account_id as string),
                });
              },

              async action({ params, request }) {
                let data = Object.fromEntries(await request.formData());
                switch (request.method) {
                  case "PATCH":
                    return await apiClient.updateAccount(
                      params.account_id as string,
                      data as unknown as PartialAccount
                    );
                  default:
                    throw new Error(`unexpected method ${request.method}`);
                }
              },

              children: [
                {
                  path: "",
                  element: <AccountSummary />,
                },
                {
                  path: "memberships",
                  element: <Memberships />,
                  loader({ params }) {
                    return defer({
                      memberships: apiClient.accountMemberships(
                        params.account_id as string
                      ),
                    });
                  },
                  async action({ params, request }) {
                    let data = Object.fromEntries(await request.formData());
                    switch (request.method) {
                      case "DELETE":
                        return await apiClient.deleteMembership(
                          data.membershipId as string
                        );
                      case "POST":
                        return await apiClient.createMembership(
                          params.account_id as string,
                          data as { user_email: string }
                        );
                      default:
                        throw new Error(`unexpected method ${request.method}`);
                    }
                  },
                },
                {
                  path: "tasks",
                  element: <TaskList />,
                  loader({ params }) {
                    return defer({
                      tasks: apiClient.accountTasks(
                        params.account_id as string
                      ),
                    });
                  },
                },
                {
                  path: "tasks/:task_id",
                  element: <TaskDetail />,
                  loader({ params }) {
                    return defer({
                      task: apiClient.task(params.task_id as string),
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
                  path: "tasks/new",
                  element: <TaskForm />,
                },
              ],
            },
            {
              path: "new",
              element: <AccountForm apiClient={apiClient} />,
            },
          ],
        },
      ],
    },
  ]);
}

export default function Router() {
  let apiClient = React.useContext(ApiClientContext);
  let router = React.useMemo(() => buildRouter(apiClient), [apiClient]);
  return <RouterProvider router={router} />;
}
