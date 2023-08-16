import { RouteObject, defer, redirect } from "react-router-dom";
import ApiClient, { PartialAccount } from "../ApiClient";
import AccountSummary from "./AccountSummary";
import AccountForm from "./AccountForm";
import AccountList from "./AccountList";

export default function accounts(
  apiClient: ApiClient,
  children: RouteObject[],
): RouteObject {
  return {
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
        path: "new",
        element: <AccountForm />,
        async action({ request }) {
          let data = Object.fromEntries(await request.formData());
          switch (request.method) {
            case "POST":
              const account = await apiClient.createAccount(
                data as unknown as PartialAccount,
              );
              return redirect(`/accounts/${account.id}`);
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },
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
              return {
                account: await apiClient.updateAccount(
                  params.account_id as string,
                  data as unknown as PartialAccount,
                ),
              };
            default:
              throw new Error(`unexpected method ${request.method}`);
          }
        },

        shouldRevalidate(args) {
          return (
            typeof args.actionResult === "object" &&
            args.actionResult !== null &&
            "account" in args.actionResult
          );
        },
        children: [
          { element: <AccountSummary />, path: "", index: true },
          ...children,
        ],
      },
    ],
  };
}
