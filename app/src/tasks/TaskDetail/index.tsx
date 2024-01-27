import {
  Await,
  useParams,
  useRouteLoaderData,
  useLoaderData,
} from "react-router-dom";
import Breadcrumb from "react-bootstrap/Breadcrumb";
import React from "react";
import Row from "react-bootstrap/Row";
import { LinkContainer } from "react-router-bootstrap";
import { Task } from "../../ApiClient";
import "@github/relative-time-element";
import { AccountBreadcrumbs } from "../../util";
import Placeholder from "react-bootstrap/Placeholder";
import TaskTitle from "./TaskTitle";
import CollectorAuthTokens from "./CollectorAuthTokens";
import ClientConfig from "./ClientConfig";
import TaskPropertyTable from "./TaskPropertyTable";

export default function TaskDetail() {
  return (
    <>
      <Breadcrumbs />
      <Row>
        <TaskTitle />
      </Row>

      <Row>
        <TaskPropertyTable />
        <ClientConfig />
        <CollectorAuthTokens />
      </Row>
    </>
  );
}

function Breadcrumbs() {
  const { task } = useLoaderData() as {
    task: Promise<Task>;
  };
  const { accountId } = useParams();

  return (
    <AccountBreadcrumbs>
      <LinkContainer to={`/accounts/${accountId}/tasks`}>
        <Breadcrumb.Item>Tasks</Breadcrumb.Item>
      </LinkContainer>
      <Breadcrumb.Item active>
        <React.Suspense fallback={<Placeholder animation="glow" xs={6} />}>
          <Await resolve={task}>{(task) => task.name}</Await>
        </React.Suspense>
      </Breadcrumb.Item>
    </AccountBreadcrumbs>
  );
}

export function WithTask({
  children,
}: {
  children: (data: Awaited<Task>) => React.ReactNode;
}) {
  const { task } = useRouteLoaderData("task") as {
    task: Promise<Task>;
  };

  return <Await resolve={task}>{children}</Await>;
}
