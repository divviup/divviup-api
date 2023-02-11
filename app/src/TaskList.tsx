import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import ListGroup from "react-bootstrap/ListGroup";
import {
  Await,
  useLoaderData,
  useAsyncValue,
  useRouteLoaderData,
} from "react-router-dom";
import { Suspense } from "react";
import { Account, Task } from "./ApiClient";
import { Alert, Button, Spinner } from "react-bootstrap";
import { LinkContainer } from "react-router-bootstrap";
import { FileEarmarkCode } from "react-bootstrap-icons";
import { VdafIcon } from "./TaskDetail";

export default function Memberships() {
  let { account } = useRouteLoaderData("account") as {
    account: Promise<Account>;
  };
  return (
    <Suspense fallback={<Spinner />}>
      <Await resolve={account}>
        <AccountDetailFull />
      </Await>
    </Suspense>
  );
}

function AccountDetailFull() {
  let account = useAsyncValue() as Account;
  let { tasks } = useLoaderData() as {
    tasks: Promise<Task[]>;
  };
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
            <LinkContainer to={`/accounts/${account.id}`}>
              <Breadcrumb.Item>{account.name}</Breadcrumb.Item>
            </LinkContainer>
            <Breadcrumb.Item active>Tasks</Breadcrumb.Item>
          </Breadcrumb>
        </Col>
      </Row>

      <Row>
        <Col>
          <h1>
            <FileEarmarkCode /> {account.name} Tasks
          </h1>
          <Suspense fallback={<Spinner />}>
            <Await resolve={tasks}>
              <TaskList />
            </Await>
          </Suspense>
        </Col>
      </Row>
      <Row className="mb-3">
        <Col>
          <LinkContainer to="new">
            <Button>New task</Button>
          </LinkContainer>
        </Col>
      </Row>
    </>
  );
}

function TaskList() {
  let tasks = useAsyncValue() as Task[];
  if (tasks.length === 0) {
    return (
      <Alert variant="warning">
        <h2>There are no tasks</h2>
      </Alert>
    );
  } else {
    return (
      <ListGroup>
        {tasks.map((task) => (
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
