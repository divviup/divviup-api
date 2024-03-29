import { AxiosError } from "axios";
import Alert from "react-bootstrap/Alert";
import { isRouteErrorResponse, useRouteError } from "react-router-dom";
import ApiClient from "../ApiClient";
import Layout from "./Layout";
import React from "react";

export default function ErrorPage({ apiClient }: { apiClient: ApiClient }) {
  const error = useRouteError();

  const doLoginRedirect =
    error instanceof AxiosError && error.response?.status === 403;
  React.useEffect(() => {
    if (doLoginRedirect) {
      apiClient.loginUrl().then((url) => {
        window.location.replace(url);
      });
    }
  }, [doLoginRedirect, apiClient]);

  if (error instanceof AxiosError) {
    switch (error.response?.status) {
      case 404: {
        return (
          <Layout error>
            <Alert variant="warning">Not Found</Alert>
          </Layout>
        );
      }

      case 500: {
        const body = error.response?.data;
        return (
          <Layout error>
            <Alert variant="danger">
              <h1>Whoops!</h1>
              <p>{body}</p>
            </Alert>
          </Layout>
        );
      }
    }
  }

  if (isRouteErrorResponse(error)) {
    return (
      <Layout error>
        <Alert variant="warning">Not Found</Alert>
      </Layout>
    );
  }

  return (
    <>
      <h1>Whoops!</h1>
      <pre>
        <code>
          {typeof error === "object" &&
          error &&
          "stack" in error &&
          typeof error.stack === "string"
            ? error.stack
            : null}
        </code>
      </pre>
    </>
  );
}
