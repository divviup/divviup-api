import { Await, useLoaderData } from "react-router-dom";
import Col from "react-bootstrap/Col";
import { Suspense } from "react";
import { Aggregator, Task } from "../../ApiClient";
import Card from "react-bootstrap/Card";
import ListGroup from "react-bootstrap/ListGroup";
import { DateTime } from "luxon";
import Row from "react-bootstrap/Row";
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
                  Successful Uploads:{" "}
                  {numberFormat.format(task.report_counter_success)}
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
                <FailedMetric
                  name="Duplicate Extension Failure"
                  counter={task.report_counter_duplicate_extension}
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

function AggregationJobMetrics({ task }: { task: Promise<Task> }) {
  return (
    <Col md="6">
      <Card className="my-3">
        <Card.Body>
          <Card.Title>Aggregation Job Metrics</Card.Title>
          <Card.Subtitle>
            <OutLink href="https://docs.divviup.org/product-documentation/operational-metrics#aggregation-job-metrics">
              View Documentation
            </OutLink>
          </Card.Subtitle>
        </Card.Body>
        <Suspense fallback={<Placeholder animation="glow" xs={2} />}>
          <Await resolve={task}>
            {(task: Task) => (
              <ListGroup variant="flush">
                <ListGroup.Item>
                  Successful Report Preparations:{" "}
                  {numberFormat.format(task.aggregation_job_counter_success)}
                </ListGroup.Item>
                <FailedMetric
                  name="Batch Collected (Helper) Failure"
                  counter={task.aggregation_job_counter_helper_batch_collected}
                />
                <FailedMetric
                  name="Report Replayed (Helper) Failure"
                  counter={task.aggregation_job_counter_helper_report_replayed}
                />
                <FailedMetric
                  name="Report Dropped (Helper) Failure"
                  counter={task.aggregation_job_counter_helper_report_dropped}
                />
                <FailedMetric
                  name="Unknown HPKE Config ID (Helper) Failure"
                  counter={
                    task.aggregation_job_counter_helper_hpke_unknown_config_id
                  }
                />
                <FailedMetric
                  name="HPKE Decryption (Helper) Failure"
                  counter={
                    task.aggregation_job_counter_helper_hpke_decrypt_failure
                  }
                />
                <FailedMetric
                  name="VDAF Preparation (Helper) Failure"
                  counter={task.aggregation_job_counter_helper_vdaf_prep_error}
                />
                <FailedMetric
                  name="Task Expiration (Helper) Failure"
                  counter={task.aggregation_job_counter_helper_task_expired}
                />
                <FailedMetric
                  name="Invalid Message (Helper) Failure"
                  counter={task.aggregation_job_counter_helper_invalid_message}
                />
                <FailedMetric
                  name="Report Too Early (Helper) Failure"
                  counter={task.aggregation_job_counter_helper_report_too_early}
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
          return (
            <Row>
              {leaderAggregator.features.includes("UploadMetrics") ? (
                <UploadMetrics task={task} />
              ) : null}
              {leaderAggregator.features.includes("AggregationJobMetrics") ? (
                <AggregationJobMetrics task={task} />
              ) : null}
            </Row>
          );
        }}
      </Await>
    </Suspense>
  );
}
