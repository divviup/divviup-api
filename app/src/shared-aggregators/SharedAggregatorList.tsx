import {
  Col,
  ListGroup,
  ListGroupItem,
  Placeholder,
  Row,
} from "react-bootstrap";
import { Await, useLoaderData } from "react-router-dom";
import { Aggregator } from "../ApiClient";
import "@github/relative-time-element";
import { Suspense } from "react";
import SharedAggregatorForm from "./SharedAggregatorForm";

export const Component = JobQueue;

export function JobQueue() {
  let { aggregators } = useLoaderData() as {
    aggregators: Promise<Aggregator[]>;
  };

  return (
    <>
      <Row style={{ height: "calc(100vh - 56px)" }}>
        <Col sm={6} style={{ overflowY: "auto", height: "100%" }}>
          <ListGroup className="list-group-flush">
            <Suspense fallback={<Placeholder animation="wave" xs={6} />}>
              <Await resolve={aggregators}>
                {(aggregators: Aggregator[]) =>
                  aggregators.length === 0 ? (
                    <ListGroupItem disabled>none</ListGroupItem>
                  ) : (
                    aggregators.map((aggregator) => (
                      <AggregatorRow
                        aggregator={aggregator}
                        key={aggregator.id}
                      />
                    ))
                  )
                }
              </Await>
            </Suspense>
          </ListGroup>
        </Col>
        <Col sm={6}>
          <SharedAggregatorForm />
        </Col>
      </Row>
    </>
  );
}

function AggregatorRow({ aggregator }: { aggregator: Aggregator }) {
  let aggWithoutAccountId = { ...aggregator } as Partial<Aggregator>;
  delete aggWithoutAccountId.account_id;
  return (
    <ListGroupItem>
      <pre>
        <code>
          {JSON.stringify(aggWithoutAccountId, null, 2).replaceAll(
            /"|,|\{|\}|  /g,
            ""
          )}
        </code>
      </pre>
    </ListGroupItem>
  );
}
