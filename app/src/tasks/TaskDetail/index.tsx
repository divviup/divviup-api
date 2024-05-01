import { Await, useParams, useLoaderData } from "react-router-dom";
import Breadcrumb from "react-bootstrap/Breadcrumb";
import React, { Suspense } from "react";
import Row from "react-bootstrap/Row";
import { LinkContainer } from "react-router-bootstrap";
import { Task } from "../../ApiClient";
import "@github/relative-time-element";
import { AccountBreadcrumbs } from "../../util";
import Placeholder from "react-bootstrap/Placeholder";
import CollectorAuthTokens from "./CollectorAuthTokens";
import Metrics from "./Metrics";
import ClientConfig from "./ClientConfig";
import TaskPropertyTable from "./TaskPropertyTable";
import { ButtonGroup, ButtonToolbar, Col } from "react-bootstrap";
import RenameTaskButton from "./RenameTaskButton";
import DisableTaskButton from "./DisableTaskButton";
import DeleteTaskButton from "./DeleteTaskButton";
import { VdafIcon } from "../VdafIcon";

export default function TaskDetail() {
  const { task } = useLoaderData() as {
    task: Promise<Task>;
  };

  return (
    <>
      <Breadcrumbs />
      <Row className="align-items-center">
        <React.Suspense>
          <Await resolve={task}>
            {(task: Task) => {
              return (
                <Col>
                  <h1>
                    <React.Suspense fallback={<Placeholder />}>
                      <Await resolve={task}>
                        {(task) => (
                          <>
                            <VdafIcon fill task={task} /> {task.name}
                          </>
                        )}
                      </Await>
                    </React.Suspense>
                  </h1>
                </Col>
              );
            }}
          </Await>
        </React.Suspense>
        <Col className="mr-auto">
          <ButtonToolbar className="float-sm-end">
            <ButtonGroup className="me-2">
              <RenameTaskButton />
            </ButtonGroup>
            <ButtonGroup>
              <DisableTaskButton />
              <DeleteTaskButton />
            </ButtonGroup>
          </ButtonToolbar>
        </Col>
      </Row>
      <Row>
        <TaskPropertyTable />
        <ClientConfig />
        <Metrics />
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
  fallback,
}: {
  children: (data: Awaited<Task>) => React.ReactNode;
  fallback?: React.ReactNode;
}) {
  const { task } = useLoaderData() as {
    task: Promise<Task>;
  };

  return (
    <Suspense fallback={fallback || <Placeholder animation="glow" xs={6} />}>
      <Await resolve={task}>{children}</Await>
    </Suspense>
  );
}
