import { Steps } from "primereact/steps";
import { Button, Card, Col, Row } from "react-bootstrap";
import { useLoaderData } from "react-router-dom";
import {
  Account,
  Aggregator,
  CollectorCredential,
  Task,
} from "../../ApiClient";
import { LinkContainer } from "react-router-bootstrap";
import AggregatorTypeSelection from "./AggregatorTypeSelection";
import InlineCollectorCredentials from "./InlineCollectorCredentials";
import { usePromise, usePromiseAll3 } from "../../util";
import css from "./index.module.css";
import { useCallback } from "react";

const STEPS = [
  { label: "Create account" },
  { label: "Set up aggregators" },
  { label: "Set up collector credentials" },
  { label: "Create your first task" },
];

function determineModel({
  account,
  collectorCredentials,
  aggregators,
}: {
  account: Account;
  collectorCredentials: CollectorCredential[];
  aggregators: Aggregator[];
}): number {
  const aggregatorStepComplete =
    account.intends_to_use_shared_aggregators === true ||
    !!aggregators.find((a) => !!a.account_id);
  const collectorCredentialStepComplete = collectorCredentials.length > 0;

  const nextStepIndex =
    [aggregatorStepComplete, collectorCredentialStepComplete, false].findIndex(
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
  const { account, collectorCredentials, aggregators, tasks } =
    useLoaderData() as {
      account: Promise<Account>;
      collectorCredentials: Promise<CollectorCredential[]>;
      aggregators: Promise<Aggregator[]>;
      tasks: Promise<Task[]>;
    };

  const loadedTasks = usePromise(tasks, []);
  const activeIndex = usePromiseAll3(
    account,
    collectorCredentials,
    aggregators,
    useCallback(
      ([account, collectorCredentials, aggregators]) =>
        determineModel({ account, collectorCredentials, aggregators }),
      [],
    ),
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
          <Steps
            className={`my-3 ${css.fixBootstrapUnderline}`}
            model={STEPS}
            activeIndex={activeIndex}
          />
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
