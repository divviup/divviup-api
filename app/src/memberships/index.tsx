import { RouteObject, defer } from "react-router-dom";
import ApiClient from "../ApiClient";
import Memberships from "./Memberships";

export default function memberships(apiClient: ApiClient): RouteObject {
  return {
    path: "memberships",
    element: <Memberships />,
    loader({ params }) {
      return defer({
        memberships: apiClient.accountMemberships(params.account_id as string),
      });
    },

    shouldRevalidate(_) {
      return true;
    },

    async action({ params, request }) {
      let data = Object.fromEntries(await request.formData());
      switch (request.method) {
        case "DELETE":
          await apiClient.deleteMembership(data.membershipId as string);
          return { deleted: data.membershipId };
        case "POST":
          return await apiClient.createMembership(
            params.account_id as string,
            data as { user_email: string }
          );
        default:
          throw new Error(`unexpected method ${request.method}`);
      }
    },
  };
}
