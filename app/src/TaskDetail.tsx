import {
  Await,
  useParams,
  useRouteLoaderData,
  useLoaderData,
  Form,
  useActionData,
  Link,
} from "react-router-dom";
import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import React, { Suspense, useCallback, useEffect, useState } from "react";
import Row from "react-bootstrap/Row";
import { LinkContainer } from "react-router-bootstrap";
import { Task, Account, Aggregator } from "./ApiClient";
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
import "@github/relative-time-element";
import { AccountBreadcrumbs } from "./util";

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
  let { task } = useLoaderData() as {
    task: Promise<Task>;
  };
  let { account_id } = useParams();

  return (
    <AccountBreadcrumbs>
      <LinkContainer to={`/accounts/${account_id}/tasks`}>
        <Breadcrumb.Item>Tasks</Breadcrumb.Item>
      </LinkContainer>
      <Breadcrumb.Item active>
        <React.Suspense fallback="...">
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
  let { task } = useRouteLoaderData("task") as {
    task: Promise<Task>;
  };

  return <Await resolve={task} children={children} />;
}

function TaskPropertyTable() {
  let { account_id } = useParams();
  let { task, leaderAggregator, helperAggregator } = useLoaderData() as {
    task: Promise<Task>;
    leaderAggregator: Promise<Aggregator>;
    helperAggregator: Promise<Aggregator>;
  };

  return (
    <Col>
      <Card className="my-3">
        <Card.Body>
          <Card.Title>Task Properties</Card.Title>
        </Card.Body>
        <ListGroup variant="flush">
          <ListGroup.Item>
            Task Id:{" "}
            <code>
              <Suspense fallback="...">
                <Await resolve={task}>{(task) => task.id}</Await>
              </Suspense>
            </code>
          </ListGroup.Item>

          <ListGroup.Item>
            Time Precision:{" "}
            <Suspense fallback="...">
              <Await resolve={task}>
                {(task) => humanizeDuration(1000 * task.time_precision_seconds)}
              </Await>
            </Suspense>
          </ListGroup.Item>
          <ListGroup.Item>
            Query Type:{" "}
            <Suspense fallback="...">
              <Await resolve={task}>
                {(task) =>
                  typeof task.max_batch_size === "number"
                    ? `Fixed maximum batch size ${task.max_batch_size}`
                    : "Time Interval"
                }
              </Await>
            </Suspense>
          </ListGroup.Item>
          <ListGroup.Item>
            Minimum Batch Size:{" "}
            <Suspense fallback="...">
              <Await resolve={task}>{(task) => task.min_batch_size}</Await>
            </Suspense>
          </ListGroup.Item>
          <ListGroup.Item>
            Expires:{" "}
            <Suspense fallback="...">
              <Await resolve={task}>
                {(task) =>
                  task.expiration
                    ? DateTime.fromISO(task.expiration)
                      .toLocal()
                      .toLocaleString(DateTime.DATETIME_SHORT)
                    : "never"
                }
              </Await>
            </Suspense>
          </ListGroup.Item>
          <ListGroup.Item>
            Leader:{" "}
            <Suspense fallback="...">
              <Await resolve={leaderAggregator}>
                {(aggregator) => (
                  <Link
                    to={`/accounts/${account_id}/aggregators/${aggregator.id}`}
                  >
                    {aggregator.name}
                  </Link>
                )}
              </Await>
            </Suspense>
          </ListGroup.Item>
          <ListGroup.Item>
            Helper:{" "}
            <Suspense fallback="...">
              <Await resolve={helperAggregator}>
                {(aggregator) => (
                  <Link
                    to={`/accounts/${account_id}/aggregators/${aggregator.id}`}
                  >
                    {aggregator.name}
                  </Link>
                )}
              </Await>
            </Suspense>
          </ListGroup.Item>
          <ListGroup.Item>
            Created:{" "}
            <Suspense fallback="...">
              <Await resolve={task}>
                {(task) =>
                  DateTime.fromISO(task.created_at)
                    .toLocal()
                    .toLocaleString(DateTime.DATETIME_SHORT)
                }
              </Await>
            </Suspense>
          </ListGroup.Item>
          <Suspense fallback="...">
            <Await resolve={task}>{(task) => <Vdaf task={task} />}</Await>
          </Suspense>
        </ListGroup>
      </Card>
    </Col>
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
  let { task } = useLoaderData() as {
    task: Promise<Task>;
  };

  return (
    <Col>
      <Card className="my-3">
        <Card.Body>
          <Card.Title>Metrics</Card.Title>
        </Card.Body>
        <ListGroup variant="flush">
          <ListGroup.Item>
            Report Count:{" "}
            <Suspense fallback="0">
              <Await resolve={task}>{(task) => task.report_count}</Await>
            </Suspense>
          </ListGroup.Item>
          <ListGroup.Item>
            Aggregate Collection Count:{" "}
            <Suspense fallback="0">
              <Await resolve={task}>
                {(task) => task.aggregate_collection_count}
              </Await>
            </Suspense>
          </ListGroup.Item>
        </ListGroup>
        <Card.Footer className="text-muted">
          Last updated{" "}
          <Suspense fallback="...">
            <Await resolve={task}>
              {(task) => (
                <relative-time datetime={task.updated_at} format="relative">
                  {DateTime.fromISO(task.updated_at)
                    .toLocal()
                    .toLocaleString(DateTime.DATETIME_SHORT)}
                </relative-time>
              )}
            </Await>
          </Suspense>
        </Card.Footer>
      </Card>
    </Col>
  );
}

function ClientConfig() {
  let { task, leaderAggregator, helperAggregator } = useLoaderData() as {
    task: Promise<Task>;
    leaderAggregator: Promise<Aggregator>;
    helperAggregator: Promise<Aggregator>;
  };
  let all = Promise.all([task, leaderAggregator, helperAggregator]);

  return (
    <React.Suspense>
      <Await resolve={all}>
        {([task, leader, helper]) => {
          const json = {
            ...task.vdaf,
            taskId: task.id,
            leader: leader.dap_url,
            helper: helper.dap_url,
            timePrecisionSeconds: task.time_precision_seconds,
          };

          return (
            <Col>
              <Card className="my-3">
                <Card.Body>
                  <Card.Title>Client Config</Card.Title>
                  Copy and paste this code to use{" "}
                  <a href="https://github.com/divviup/divviup-ts">divviup-ts</a>
                  <pre>
                    <code className="my-3">
                      import DAPClient from "@divviup/dap";{"\n"}const client =
                      new DAPClient({JSON.stringify(json, null, 2)});
                    </code>
                  </pre>
                </Card.Body>
              </Card>
            </Col>
          );
        }}
      </Await>
    </React.Suspense>
  );
}
