import { useFetcher } from "react-router-dom";
import Col from "react-bootstrap/Col";
import React from "react";
import { Aggregator, CollectorAuthToken } from "../../ApiClient";
import Button from "react-bootstrap/Button";
import Card from "react-bootstrap/Card";
import ListGroup from "react-bootstrap/ListGroup";
import { CopyCode, useLoaderPromise } from "../../util";
import { Badge } from "react-bootstrap";

export default function CollectorAuthTokens() {
  const fetcher = useFetcher();
  const callback = React.useCallback(() => {
    if (fetcher.state === "idle" && !fetcher.data)
      fetcher.load("collector_auth_tokens");
  }, [fetcher]);

  const leader = useLoaderPromise<Aggregator | null>("leaderAggregator", null);
  if (!leader || leader.features.includes("TokenHash")) {
    return <></>;
  }

  if (fetcher.data) {
    const { collectorAuthTokens } = fetcher.data as {
      collectorAuthTokens: CollectorAuthToken[];
    };

    return (
      <Col md="6">
        <Card className="my-3">
          <Card.Body>
            <Card.Title>Collector Auth Tokens</Card.Title>
          </Card.Body>
          <ListGroup variant="flush">
            {collectorAuthTokens.map((collectorAuthToken) => (
              <ListGroup.Item key={collectorAuthToken.token}>
                <Badge bg="primary" pill className="mx-1">
                  {collectorAuthToken.type}
                </Badge>
                <CopyCode code={collectorAuthToken.token} />
              </ListGroup.Item>
            ))}
          </ListGroup>
        </Card>
      </Col>
    );
  } else {
    return (
      <Col md="6">
        <Card className="my-3">
          <Card.Body>
            <Card.Title>Collector Auth Tokens</Card.Title>
            <Button
              variant="warning"
              onClick={callback}
              disabled={fetcher.state === "loading"}
            >
              Reveal
            </Button>
          </Card.Body>
        </Card>
      </Col>
    );
  }
}
