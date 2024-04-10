import { Await, useLoaderData } from "react-router-dom";
import Col from "react-bootstrap/Col";
import { Suspense } from "react";
import { Aggregator, Task } from "../../ApiClient";
import Card from "react-bootstrap/Card";
import ListGroup from "react-bootstrap/ListGroup";
import { DateTime } from "luxon";
import Placeholder from "react-bootstrap/Placeholder";
import { OutLink, numberFormat } from "../../util";

function FailedMetric({ name, counter }: { name: string; counter: number }) {
  if (counter > 0) {
    return (
      <ListGroup.Item>
        {name}: {numberFormat.format(counter)}
      </ListGroup.Item>
    );
  } else {
    return null;
  }
}

function UploadMetrics({ task }: { task: Promise<Task> }) {
  return (
    <Col md="6">
      <Card className="my-3">
        <Card.Body>
          <Card.Title>Upload Metrics</Card.Title>
          <Card.Subtitle>
            <OutLink href="https://docs.divviup.org/product-documentation/operational-metrics#upload-metrics">
              View Documentation
            </OutLink>
          </Card.Subtitle>
        </Card.Body>
        <Suspense fallback={<Placeholder animation="glow" xs={2} />}>
          <Await resolve={task}>
            {(task: Task) => (
              <ListGroup variant="flush">
                <ListGroup.Item>
                  Successful Uploads: {task.report_counter_success}
                </ListGroup.Item>
                <FailedMetric
                  name="Interval Collected Failure"
                  counter={task.report_counter_interval_collected}
                />
                <FailedMetric
                  name="Decode Failure"
                  counter={task.report_counter_decode_failure}
                />
                <FailedMetric
                  name="Decrypt Failure"
                  counter={task.report_counter_decrypt_failure}
                />
                <FailedMetric
                  name="Report Expired Failure"
                  counter={task.report_counter_expired}
                />
                <FailedMetric
                  name="Outdated Key Failure"
                  counter={task.report_counter_outdated_key}
                />
                <FailedMetric
                  name="Report Too Early Failure"
                  counter={task.report_counter_too_early}
                />
                <FailedMetric
                  name="Task Expired Failure"
                  counter={task.report_counter_task_expired}
                />
              </ListGroup>
            )}
          </Await>
        </Suspense>
        <Card.Footer className="text-muted">
          Last updated{" "}
          <Suspense fallback={<Placeholder animation="glow" xs={1} />}>
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

export default function Metrics() {
  const { task, leaderAggregator } = useLoaderData() as {
    task: Promise<Task>;
    leaderAggregator: Promise<Aggregator>;
  };

  return (
    <Suspense fallback={<Placeholder animation="glow" xs={2} />}>
      <Await resolve={leaderAggregator}>
        {(leaderAggregator) => {
          if (leaderAggregator.features.includes("UploadMetrics")) {
            return <UploadMetrics task={task} />;
          }
        }}
      </Await>
    </Suspense>
  );
}
