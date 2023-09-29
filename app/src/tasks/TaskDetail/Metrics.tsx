import { Await, useLoaderData } from "react-router-dom";
import Col from "react-bootstrap/Col";
import { Suspense } from "react";
import { Task } from "../../ApiClient";
import Card from "react-bootstrap/Card";
import ListGroup from "react-bootstrap/ListGroup";
import { DateTime } from "luxon";
import Placeholder from "react-bootstrap/Placeholder";

export default function Metrics() {
  const { task } = useLoaderData() as {
    task: Promise<Task>;
  };

  return (
    <Col md="6">
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
          <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
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
