import Breadcrumb from "react-bootstrap/Breadcrumb";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import Button from "react-bootstrap/Button";
import { AccountBreadcrumbs, WithAccount } from "../util";
import { CloudUpload } from "react-bootstrap-icons";
import { Suspense } from "react";
import { LinkContainer } from "react-router-bootstrap";
import { Await, useLoaderData } from "react-router-dom";
import { Aggregator } from "../ApiClient";
import { ListGroup } from "react-bootstrap";
import D from "../logo/color/svg/small.svg";
import Placeholder from "react-bootstrap/Placeholder";

export default function Aggregators() {
  return (
    <>
      <Breadcrumbs />
      <Row>
        <Col>
          <h1>
            <CloudUpload />{" "}
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <WithAccount>{(account) => account.name}</WithAccount>
            </Suspense>{" "}
            Aggregators
          </h1>
        </Col>
      </Row>
      <Row className="mb-3">
        <Col>
          <LinkContainer to="new">
            <Button>New aggregator</Button>
          </LinkContainer>
        </Col>
      </Row>
      <Row>
        <Col>
          <AggregatorList />
        </Col>
      </Row>
    </>
  );
}

function Breadcrumbs() {
  return (
    <AccountBreadcrumbs>
      <Breadcrumb.Item active>Aggregators</Breadcrumb.Item>
    </AccountBreadcrumbs>
  );
}

function AggregatorList() {
  const { aggregators } = useLoaderData() as {
    aggregators: Promise<Aggregator[]>;
  };
  return (
    <ListGroup>
      <Suspense>
        <Await resolve={aggregators}>
          {(aggregators: Aggregator[]) =>
            aggregators.map((aggregator) => (
              <LinkContainer key={aggregator.id} to={aggregator.id}>
                <ListGroup.Item action>
                  {" "}
                  {aggregator.is_first_party ? (
                    <img
                      src={D}
                      style={{ height: "1em", marginTop: "-0.2em" }}
                    />
                  ) : (
                    <CloudUpload />
                  )}{" "}
                  {aggregator.name}
                </ListGroup.Item>
              </LinkContainer>
            ))
          }
        </Await>
      </Suspense>
    </ListGroup>
  );
}
