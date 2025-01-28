import { Await, useParams, useLoaderData, Link } from "react-router";
import Col from "react-bootstrap/Col";
import { Suspense } from "react";
import { Task, Aggregator, CollectorCredential } from "../../ApiClient";
import humanizeDuration from "humanize-duration";
import Card from "react-bootstrap/Card";
import ListGroup from "react-bootstrap/ListGroup";
import { DateTime } from "luxon";
import { CopyCode } from "../../util";
import Placeholder from "react-bootstrap/Placeholder";
import Vdaf from "./Vdaf";

export default function TaskPropertyTable() {
  const { accountId } = useParams();
  const { task, leaderAggregator, helperAggregator, collectorCredential } =
    useLoaderData() as {
      task: Promise<Task>;
      leaderAggregator: Promise<Aggregator>;
      helperAggregator: Promise<Aggregator>;
      collectorCredential: Promise<CollectorCredential>;
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
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <Await resolve={task}>
                {(task) => <CopyCode code={task.id} />}
              </Await>
            </Suspense>
          </ListGroup.Item>

          <ListGroup.Item>
            Time Precision:{" "}
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <Await resolve={task}>
                {(task) => humanizeDuration(1000 * task.time_precision_seconds)}
              </Await>
            </Suspense>
          </ListGroup.Item>
          <Suspense
            fallback={
              <ListGroup.Item>
                <Placeholder animation="glow" xs={6} />
              </ListGroup.Item>
            }
          >
            <Await resolve={task}>
              {(task) => {
                let queryType;
                let maxBatchSize;
                let batchTimeWindowSize;

                if (typeof task.max_batch_size === "number") {
                  maxBatchSize = (
                    <ListGroup.Item>
                      Maximum Batch Size: {task.max_batch_size}
                    </ListGroup.Item>
                  );

                  if (typeof task.batch_time_window_size_seconds === "number") {
                    queryType = "Time-bucketed Fixed Size";

                    batchTimeWindowSize = (
                      <ListGroup.Item>
                        Batch Time Window Size:{" "}
                        {humanizeDuration(
                          1000 * task.batch_time_window_size_seconds,
                        )}
                      </ListGroup.Item>
                    );
                  } else {
                    queryType = "Fixed Size";
                  }
                } else {
                  queryType = "Time Interval";
                }

                return (
                  <>
                    <ListGroup.Item>Query Type: {queryType}</ListGroup.Item>
                    {batchTimeWindowSize}
                    {maxBatchSize}
                  </>
                );
              }}
            </Await>
          </Suspense>
          <ListGroup.Item>
            Minimum Batch Size:{" "}
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <Await resolve={task}>{(task) => task.min_batch_size}</Await>
            </Suspense>
          </ListGroup.Item>
          <ListGroup.Item>
            Expires:{" "}
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
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
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <Await resolve={leaderAggregator}>
                {(aggregator) => (
                  <Link
                    to={`/accounts/${accountId}/aggregators/${aggregator.id}`}
                  >
                    {aggregator.name}
                  </Link>
                )}
              </Await>
            </Suspense>
          </ListGroup.Item>
          <ListGroup.Item>
            Helper:{" "}
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <Await resolve={helperAggregator}>
                {(aggregator) => (
                  <Link
                    to={`/accounts/${accountId}/aggregators/${aggregator.id}`}
                  >
                    {aggregator.name}
                  </Link>
                )}
              </Await>
            </Suspense>
          </ListGroup.Item>
          <ListGroup.Item>
            Created:{" "}
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <Await resolve={task}>
                {(task) =>
                  DateTime.fromISO(task.created_at)
                    .toLocal()
                    .toLocaleString(DateTime.DATETIME_SHORT)
                }
              </Await>
            </Suspense>
          </ListGroup.Item>
          <ListGroup.Item>
            Collector Crededential:{" "}
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <Await resolve={collectorCredential}>
                {(collectorCredential) => (
                  <Link to={`/accounts/${accountId}/collector_credentials`}>
                    {collectorCredential.name}
                  </Link>
                )}
              </Await>
            </Suspense>
          </ListGroup.Item>
          <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
            <Await resolve={task}>{(task) => <Vdaf task={task} />}</Await>
          </Suspense>
        </ListGroup>
      </Card>
    </Col>
  );
}
