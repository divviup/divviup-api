import { Await, useLoaderData, useParams } from "react-router-dom";
import { Aggregator } from "./ApiClient";
import { AccountBreadcrumbs } from "./util";
import { LinkContainer } from "react-router-bootstrap";
import Breadcrumb from "react-bootstrap/Breadcrumb";
import React, { Suspense } from "react";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import { CloudUpload } from "react-bootstrap-icons";
import Table from "react-bootstrap/Table";
import D from "./logo/color/svg/small.svg";

function Breadcrumbs() {
  let { aggregator } = useLoaderData() as {
    aggregator: Promise<Aggregator>;
  };
  let { account_id } = useParams();

  return (
    <AccountBreadcrumbs>
      <LinkContainer to={`/accounts/${account_id}/aggregators`}>
        <Breadcrumb.Item>Aggregators</Breadcrumb.Item>
      </LinkContainer>
      <Breadcrumb.Item active>
        <React.Suspense fallback="...">
          <Await resolve={aggregator}>{(aggregator) => aggregator.name}</Await>
        </React.Suspense>
      </Breadcrumb.Item>
    </AccountBreadcrumbs>
  );
}

export default function AggregatorDetail() {
  let { aggregator } = useLoaderData() as {
    aggregator: Promise<Aggregator>;
  };

  return (
    <>
      <Breadcrumbs />
      <Row>
        <Col>
          <h1>
            <Suspense
              fallback={
                <>
                  <CloudUpload /> {" ..."}
                </>
              }
            >
              <Await resolve={aggregator}>
                {(aggregator) => (
                  <>
                    {aggregator.is_first_party ? (
                      <img
                        src={D}
                        style={{ height: "1em", marginTop: "-0.2em" }}
                      />
                    ) : (
                      <CloudUpload />
                    )}{" "}
                    {aggregator.name}
                  </>
                )}
              </Await>
            </Suspense>
          </h1>
        </Col>
      </Row>

      <Row>
        <Col>
          <AggregatorPropertyTable />
        </Col>
      </Row>
    </>
  );
}

function AggregatorPropertyTable() {
  return (
    <Table striped bordered className="overflow-auto" responsive>
      <tbody>
        <TableRow label="Name" value="name" />
        <TableRow label="DAP url" value="dap_url" />
        <TableRow label="API url" value="api_url" />
        <TableRow label="Role" value="role" />
      </tbody>
    </Table>
  );
}

function TableRow({
  label,
  value,
}: {
  label: React.ReactNode;
  value: keyof Aggregator | ((data: Aggregator) => React.ReactNode);
}) {
  return (
    <tr>
      <td>{label}</td>
      <td>
        <WithAggregator>
          {typeof value === "string"
            ? (aggregator) => aggregator[value]
            : value}
        </WithAggregator>
      </td>
    </tr>
  );
}

export function WithAggregator({
  children,
}: {
  children: (data: Awaited<Aggregator>) => React.ReactNode;
}) {
  let { aggregator } = useLoaderData() as {
    aggregator: Promise<Aggregator>;
  };

  return (
    <Suspense fallback="...">
      <Await resolve={aggregator} children={children} />
    </Suspense>
  );
}
