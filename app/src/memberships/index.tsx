import { RouteObject } from "react-router";
import ApiClient from "../ApiClient";
import Memberships from "./Memberships";

export default function memberships(apiClient: ApiClient): RouteObject {
  return {
    path: "memberships",
    element: <Memberships />,
    loader({ params }) {
      return {
        memberships: apiClient.accountMemberships(params.accountId as string),
      };
    },

    shouldRevalidate(_) {
      return true;
    },

    async action({ params, request }) {
      const data = Object.fromEntries(await request.formData());
      switch (request.method) {
        case "DELETE":
          await apiClient.deleteMembership(data.membershipId as string);
          return { deleted: data.membershipId };
        case "POST":
          return await apiClient.createMembership(
            params.accountId as string,
            data as { user_email: string },
          );
        default:
          throw new Error(`unexpected method ${request.method}`);
      }
    },
  };
}
