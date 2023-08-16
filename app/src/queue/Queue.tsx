import {
  Breadcrumb,
  Col,
  ListGroup,
  ListGroupItem,
  Nav,
  Row,
} from "react-bootstrap";
import { CheckSquare, Stopwatch, XCircle } from "react-bootstrap-icons";
import { Outlet, useLoaderData, useRevalidator } from "react-router";
import { QueueJob } from "../ApiClient";
import useInterval from "use-interval";
import { LinkContainer } from "react-router-bootstrap";
import { useSearchParams } from "react-router-dom";
import "@github/relative-time-element";
import { DateTime } from "luxon";

export const Component = JobQueue;

function TabLink({ search, text }: { search: string; text: string }) {
  const [params] = useSearchParams();
  const active = params.toString() === search;
  return (
    <Nav.Item>
      <LinkContainer to={{ search }}>
        <Nav.Link eventKey={text} active={active}>
          {text}
        </Nav.Link>
      </LinkContainer>
    </Nav.Item>
  );
}

export function JobQueue() {
  const revalidator = useRevalidator();

  useInterval(() => {
    if (revalidator.state === "idle") {
      revalidator.revalidate();
    }
  }, 1000);

  const queue = useLoaderData() as QueueJob[];

  return (
    <>
      <Row>
        <Col>
          <Breadcrumb>
            <LinkContainer to="/">
              <Breadcrumb.Item>Home</Breadcrumb.Item>
            </LinkContainer>
            <LinkContainer to="/admin/queue">
              <Breadcrumb.Item>Queue</Breadcrumb.Item>
            </LinkContainer>
          </Breadcrumb>
        </Col>
      </Row>

      <Row>
        <Col>
          <Nav variant="tabs">
            <TabLink search="" text="All" />
            <TabLink search="status=success" text="Success" />
            <TabLink search="status=failed" text="Failed" />
            <TabLink search="status=pending" text="Pending" />
          </Nav>

          <ListGroup className="list-group-flush">
            {queue.length === 0 ? (
              <ListGroupItem disabled>none</ListGroupItem>
            ) : null}
            {queue.map((job) => (
              <Job job={job} key={job.id} />
            ))}
          </ListGroup>
        </Col>
        <Outlet />
      </Row>
    </>
  );
}

function JobStatus({ job }: { job: QueueJob }) {
  switch (job.status) {
    case "Success":
      return <CheckSquare />;
    case "Failed":
      return <XCircle />;
    case "Pending":
      if (job.failure_count > 0) {
        return (
          <>
            <XCircle /> {job.failure_count}
          </>
        );
      } else {
        return <Stopwatch />;
      }
  }
}

function Job({ job }: { job: QueueJob }) {
  return (
    <LinkContainer to={`/admin/queue/${job.id}`}>
      <ListGroupItem
        action
        className="d-flex justify-content-between align-items-start"
      >
        <>
          <span>
            <JobStatus job={job} /> {job.job.type}
          </span>
          <small>
            <relative-time datetime={job.updated_at} format="relative">
              {DateTime.fromISO(job.updated_at)
                .toLocal()
                .toLocaleString(DateTime.DATETIME_SHORT)}
            </relative-time>
          </small>
        </>
      </ListGroupItem>
    </LinkContainer>
  );
}
