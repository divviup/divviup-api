import { Steps } from "primereact/steps";

import { Button, Card, Col, Row } from "react-bootstrap";
import { useLoaderData } from "react-router-dom";
import { Account, Aggregator, HpkeConfig, Task } from "../../ApiClient";
import React from "react";
import { LinkContainer } from "react-router-bootstrap";
import AggregatorTypeSelection from "./AggregatorTypeSelection";
import InlineCollectorCredentials from "./InlineCollectorCredentials";
import { usePromise, usePromiseAll } from "../../util";

const STEPS = [
  { label: "Create account" },
  { label: "Set up aggregators" },
  { label: "Set up collector credentials" },
  { label: "Create your first task" },
];

function determineModel({
  account,
  hpkeConfigs,
  aggregators,
}: {
  account: Account;
  hpkeConfigs: HpkeConfig[];
  aggregators: Aggregator[];
}): number {
  const aggregatorStepComplete =
    account.intends_to_use_shared_aggregators === true ||
    !!aggregators.find((a) => !!a.account_id);
  const hpkeConfigStepComplete = hpkeConfigs.length > 0;

  const nextStepIndex =
    [aggregatorStepComplete, hpkeConfigStepComplete, false].findIndex(
      (x) => !x,
    ) + 1;

  return nextStepIndex;
}
function NextInner({ activeIndex }: { activeIndex: number | undefined }) {
  switch (activeIndex) {
    case 1:
      return <AggregatorTypeSelection />;
    case 2:
      return <InlineCollectorCredentials />;
    case 3:
      return <NewTaskInstructions />;
    default:
      return <></>;
  }
}

export default function NextSteps() {
  const { account, hpkeConfigs, aggregators, tasks } = useLoaderData() as {
    account: Promise<Account>;
    hpkeConfigs: Promise<HpkeConfig[]>;
    aggregators: Promise<Aggregator[]>;
    tasks: Promise<Task[]>;
  };

  const loadedTasks = usePromise(tasks, []);
  const activeIndex = usePromiseAll(
    [account, hpkeConfigs, aggregators],
    ([account, hpkeConfigs, aggregators]) =>
      determineModel({ account, hpkeConfigs, aggregators }),
    1,
  );

  if (loadedTasks.length > 0) return <></>;

  return (
    <Row className="justify-content-md-center">
      <Col sm="10">
        <Card className="p-3 my-3 shadow">
          <h3>
            Set up your Divvi Up in just a few steps. Here’s what you can do
            next:
          </h3>
          <Steps className="my-3" model={STEPS} activeIndex={activeIndex} />
          <hr />
          <NextInner activeIndex={activeIndex} />
        </Card>
      </Col>
    </Row>
  );
}

function NewTaskInstructions() {
  return (
    <LinkContainer to="tasks/new">
      <Button>Create your first task</Button>
    </LinkContainer>
  );
}
