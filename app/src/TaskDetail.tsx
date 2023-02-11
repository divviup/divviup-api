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
import { Task, Account } from "./ApiClient";
import { Button, Card, ListGroup, Spinner } from "react-bootstrap";
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
import FormControl from "react-bootstrap/FormControl";
import InputGroup from "react-bootstrap/InputGroup";
import { DateTime } from "luxon";

function TaskTitle({ task }: { task: Task }) {
  let [isEditingName, setIsEditingName] = useState(false);
  let actionData = useActionData();
  useEffect(() => {
    if (actionData) setIsEditingName(false);
  }, [actionData]);
  let edit = useCallback(() => setIsEditingName(true), [setIsEditingName]);
  if (isEditingName) {
    return (
      <>
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
      </>
    );
  } else {
    return (
      <>
        <Row>
          <Col xs="11">
            <VdafIcon fill task={task} /> {task.name}
          </Col>
          <Col>
            <Button onClick={edit}>
              <Pencil className="text-end" />
            </Button>
          </Col>
        </Row>
      </>
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
  console.log(task.vdaf.type);
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

export default function TaskDetail() {
  let { account_id } = useParams();
  let { account } = useRouteLoaderData("account") as {
    account: Promise<Account>;
  };

  let { task } = useLoaderData() as { task: Promise<Task> };

  return (
    <>
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
      <Row>
        <Col>
          <React.Suspense fallback={<Spinner />}>
            <Await resolve={task}>
              {(task) => (
                <>
                  <Card>
                    <Card.Body>
                      <Card.Title>
                        <TaskTitle task={task} />
                      </Card.Title>
                    </Card.Body>
                    <ListGroup variant="flush">
                      <ListGroup.Item>Partner: {task.partner}</ListGroup.Item>
                      <ListGroup.Item>
                        Time Precision:{" "}
                        {humanizeDuration(1000 * task.time_precision_seconds)}
                      </ListGroup.Item>
                      <ListGroup.Item>
                        Minimum Batch Size: {task.min_batch_size}
                      </ListGroup.Item>
                      <ListGroup.Item>
                        Report Count: {task.report_count || 0}
                      </ListGroup.Item>
                      <ListGroup.Item>
                        Aggregate Collection Count:{" "}
                        {task.aggregate_collection_count || 0}
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
                        Created:{" "}
                        {DateTime.fromISO(task.created_at)
                          .toLocal()
                          .toLocaleString(DateTime.DATETIME_SHORT)}
                      </ListGroup.Item>
                      <Vdaf task={task} />
                    </ListGroup>
                  </Card>
                </>
              )}
            </Await>
          </React.Suspense>
        </Col>
      </Row>
    </>
  );
}
