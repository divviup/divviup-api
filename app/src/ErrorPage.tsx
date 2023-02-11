import { AxiosError } from "axios";
import Alert from "react-bootstrap/Alert";
import { isRouteErrorResponse, useRouteError } from "react-router";
import ApiClient from "./ApiClient";
import Layout from "./Layout";

export default function ErrorPage({ apiClient }: { apiClient: ApiClient }) {
  const error = useRouteError();

  if (error instanceof AxiosError) {
    switch (error.response?.status) {
      case 403:
        if (!apiClient.isLoggedIn()) {
          window.location.href = apiClient.loginUrl();
        }
        break;
      case 404:
        return (
          <Layout>
            <Alert variant="error">Not Found</Alert>
          </Layout>
        );
      case 500:
        return (
          <Layout>
            <Alert variant="error">{JSON.stringify(error)}</Alert>
          </Layout>
        );
    }
  }

  if (isRouteErrorResponse(error)) {
    return <Alert variant="error">{JSON.stringify(error)}</Alert>;
  }

  return (
    <>
      <h1>Whoops!</h1>
      <h2>{error as unknown as string}</h2>
      <pre>
        <code>{JSON.stringify(error, null, 2)}</code>
      </pre>
    </>
  );
}
