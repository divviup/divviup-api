import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import ListGroup from "react-bootstrap/ListGroup";
import { Await, useLoaderData, useAsyncValue } from "react-router-dom";
import { Suspense } from "react";
import { Task } from "../ApiClient";
import { Alert, Button, Spinner } from "react-bootstrap";
import { LinkContainer } from "react-router-bootstrap";
import { FileEarmarkCode } from "react-bootstrap-icons";
import { VdafIcon } from "./VdafIcon";
import { AccountBreadcrumbs, WithAccount } from "../util";
import Placeholder from "react-bootstrap/Placeholder";

function Breadcrumbs() {
  return (
    <AccountBreadcrumbs>
      <Breadcrumb.Item active>Tasks</Breadcrumb.Item>
    </AccountBreadcrumbs>
  );
}

export default function AccountDetailFull() {
  const { tasks } = useLoaderData() as {
    tasks: Promise<Task[]>;
  };
  return (
    <>
      <Breadcrumbs />
      <Row>
        <Col>
          <h1>
            <FileEarmarkCode />{" "}
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <WithAccount>{(account) => account.name}</WithAccount>
            </Suspense>{" "}
            Tasks
          </h1>
        </Col>
      </Row>
      <Row className="mb-3">
        <Col>
          <LinkContainer to="new">
            <Button>New task</Button>
          </LinkContainer>
        </Col>
      </Row>
      <Row>
        <Col>
          <Suspense fallback={<Spinner />}>
            <Await resolve={tasks}>
              <TaskList />
            </Await>
          </Suspense>
        </Col>
      </Row>
    </>
  );
}

function TaskList() {
  const tasks = useAsyncValue() as Task[];
  if (tasks.length === 0) {
    return (
      <Alert variant="warning">
        <h2>There are no tasks</h2>
      </Alert>
    );
  } else {
    return (
      <ListGroup>
        {tasks
          .sort((a, b) => a.name.localeCompare(b.name))
          .map((task) => (
            <LinkContainer key={task.id} to={task.id}>
              <ListGroup.Item action>
                <VdafIcon task={task} />
                {task.name}
              </ListGroup.Item>
            </LinkContainer>
          ))}
      </ListGroup>
    );
  }
}
