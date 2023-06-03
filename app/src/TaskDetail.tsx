import {
  Await,
  useParams,
  useRouteLoaderData,
  useLoaderData,
  Form,
  useActionData,
} from "react-router-dom";
import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import React, { useCallback, useEffect, useState } from "react";
import Row from "react-bootstrap/Row";
import { LinkContainer } from "react-router-bootstrap";
import { Task, Account, TaskMetrics } from "./ApiClient";
import humanizeDuration from "humanize-duration";
import {
  FileEarmarkBarGraph,
  FileEarmarkBarGraphFill,
  FileEarmarkBinary,
  FileEarmarkBinaryFill,
  FileEarmarkPlus,
  FileEarmarkPlusFill,
  Pencil,
  PencilFill,
} from "react-bootstrap-icons";
import Button from "react-bootstrap/Button";
import Card from "react-bootstrap/Card";
import ListGroup from "react-bootstrap/ListGroup";
import Spinner from "react-bootstrap/Spinner";
import FormControl from "react-bootstrap/FormControl";
import InputGroup from "react-bootstrap/InputGroup";
import { DateTime } from "luxon";
import Highlight from "react-highlight";
import "highlight.js/styles/googlecode.css";

function TaskTitle() {
  let { task } = useLoaderData() as {
    task: Promise<Task>;
  };

  let [isEditingName, setIsEditingName] = useState(false);
  let actionData = useActionData();
  useEffect(() => {
    if (actionData) setIsEditingName(false);
  }, [actionData]);
  let edit = useCallback(() => setIsEditingName(true), [setIsEditingName]);
  if (isEditingName) {
    return (
      <React.Suspense>
        <Await resolve={task}>
          {(task: Task) => (
            <Row>
              <Col xs="11">
                <Form method="patch">
                  <InputGroup hasValidation>
                    <InputGroup.Text id="inputGroupPrepend">
                      <VdafIcon fill task={task} />
                    </InputGroup.Text>
                    <FormControl
                      type="text"
                      name="name"
                      defaultValue={task.name}
                      required
                    />
                  </InputGroup>
                </Form>
              </Col>
              <Col>
                <Button variant="primary" type="submit">
                  <PencilFill />
                </Button>
              </Col>
            </Row>
          )}
        </Await>
      </React.Suspense>
    );
  } else {
    return (
      <React.Suspense>
        <Await resolve={task}>
          {(task: Task) => {
            return (
              <Row>
                <Col xs="10">
                  <h1>
                    <VdafIcon fill task={task} /> {task.name}
                  </h1>
                </Col>
                <Col>
                  <Button onClick={edit}>
                    <Pencil className="text-end" /> Edit Title
                  </Button>
                </Col>
              </Row>
            );
          }}
        </Await>
      </React.Suspense>
    );
  }
}

function Vdaf({ task }: { task: Task }) {
  switch (task.vdaf.type) {
    case "sum":
      return (
        <ListGroup.Item>
          Sum maximum value: {Math.pow(2, task.vdaf.bits)}
        </ListGroup.Item>
      );
    case "histogram":
      return (
        <>
          <ListGroup.Item>
            Histogram Buckets: {task.vdaf.buckets.join(", ")}
          </ListGroup.Item>
        </>
      );

    case "count":
      return <ListGroup.Item>Count</ListGroup.Item>;
  }
}

export function VdafIcon({
  task,
  fill = false,
}: {
  task: Task;
  fill?: boolean;
}) {
  switch (task.vdaf.type.toLowerCase()) {
    case "sum":
      return fill ? <FileEarmarkPlusFill /> : <FileEarmarkPlus />;
    case "histogram":
      return fill ? <FileEarmarkBarGraphFill /> : <FileEarmarkBarGraph />;
    case "count":
      return fill ? <FileEarmarkBinaryFill /> : <FileEarmarkBinary />;
    default:
      return <></>;
  }
}

function Breadcrumbs() {
  let { account_id } = useParams();
  let { account } = useRouteLoaderData("account") as {
    account: Promise<Account>;
  };

  let { task } = useLoaderData() as {
    task: Promise<Task>;
  };

  return (
    <Row>
      <Col>
        <Breadcrumb>
          <LinkContainer to="/">
            <Breadcrumb.Item>Home</Breadcrumb.Item>
          </LinkContainer>
          <LinkContainer to="/accounts">
            <Breadcrumb.Item>Accounts</Breadcrumb.Item>
          </LinkContainer>
          <LinkContainer to={`/accounts/${account_id}`}>
            <Breadcrumb.Item>
              <React.Suspense fallback={<span>...</span>}>
                <Await resolve={account}>{(account) => account.name}</Await>
              </React.Suspense>
            </Breadcrumb.Item>
          </LinkContainer>
          <LinkContainer to={`/accounts/${account_id}/tasks`}>
            <Breadcrumb.Item>Tasks</Breadcrumb.Item>
          </LinkContainer>
          <Breadcrumb.Item active>
            <React.Suspense fallback={<span>...</span>}>
              <Await resolve={task}>{(task) => task.name}</Await>
            </React.Suspense>
          </Breadcrumb.Item>
        </Breadcrumb>
      </Col>
    </Row>
  );
}

function TaskPropertyTable() {
  let { task } = useLoaderData() as {
    task: Promise<Task>;
  };

  return (
    <React.Suspense fallback={<Spinner />}>
      <Await resolve={task}>
        {(task) => (
          <Col>
            <Card className="my-3">
              <Card.Body>
                <Card.Title>Task Properties</Card.Title>
              </Card.Body>
              <ListGroup variant="flush">
                <ListGroup.Item>
                  Task Id: <code>{task.id}</code>
                </ListGroup.Item>

                <ListGroup.Item>
                  Time Precision:{" "}
                  {humanizeDuration(1000 * task.time_precision_seconds)}
                </ListGroup.Item>
                <ListGroup.Item>
                  Query Type:{" "}
                  {typeof task.max_batch_size === "number"
                    ? `Fixed maximum batch size ${task.max_batch_size}`
                    : "Time Interval"}
                </ListGroup.Item>
                <ListGroup.Item>
                  Minimum Batch Size: {task.min_batch_size}
                </ListGroup.Item>
                <ListGroup.Item>
                  Expires:{" "}
                  {task.expiration
                    ? DateTime.fromISO(task.expiration)
                        .toLocal()
                        .toLocaleString(DateTime.DATETIME_SHORT)
                    : "never"}
                </ListGroup.Item>
                <ListGroup.Item>
                  Leader: <code>{task.leader_url}</code>
                </ListGroup.Item>
                <ListGroup.Item>
                  Helper: <code>{task.helper_url}</code>
                </ListGroup.Item>
                <ListGroup.Item>
                  Created:{" "}
                  {DateTime.fromISO(task.created_at)
                    .toLocal()
                    .toLocaleString(DateTime.DATETIME_SHORT)}
                </ListGroup.Item>
                <Vdaf task={task} />
              </ListGroup>
            </Card>
          </Col>
        )}
      </Await>
    </React.Suspense>
  );
}

export default function TaskDetail() {
  return (
    <>
      <Breadcrumbs />
      <Row>
        <TaskTitle />
      </Row>

      <Row>
        <TaskPropertyTable />
        <Metrics />
        <ClientConfig />
      </Row>
    </>
  );
}

function Metrics() {
  let { metrics } = useLoaderData() as {
    metrics: Promise<TaskMetrics>;
  };

  return (
    <React.Suspense>
      <Await resolve={metrics}>
        {(metrics: TaskMetrics) => (
          <Col>
            <Card className="my-3">
              <Card.Body>
                <Card.Title>Metrics</Card.Title>
              </Card.Body>
              <ListGroup variant="flush">
                <ListGroup.Item>Report Count: {metrics.reports}</ListGroup.Item>
                <ListGroup.Item>
                  Aggregate Collection Count: {metrics.report_aggregations}
                </ListGroup.Item>
              </ListGroup>
            </Card>
          </Col>
        )}
      </Await>
    </React.Suspense>
  );
}

function ClientConfig() {
  let { task } = useLoaderData() as {
    task: Promise<Task>;
  };

  return (
    <React.Suspense>
      <Await resolve={task}>
        {(task: Task) => {
          const json = {
            ...task.vdaf,
            taskId: task.id,
            leader: task.leader_url,
            helper: task.helper_url,
            timePrecisionSeconds: task.time_precision_seconds,
          };

          return (
            <Col>
              <Card className="my-3">
                <Card.Body>
                  <Card.Title>Client Config</Card.Title>
                  Copy and paste this code to use{" "}
                  <a href="https://github.com/divviup/divviup-ts">divviup-ts</a>
                  <Highlight className="js">
                    <pre>
                      <code className="my-3">
                        import DAPClient from "@divviup/dap";{"\n"}const client
                        = new DAPClient({JSON.stringify(json, null, 2)});
                      </code>
                    </pre>
                  </Highlight>
                </Card.Body>
              </Card>
            </Col>
          );
        }}
      </Await>
    </React.Suspense>
  );
}
