import {
  Badge,
  Button,
  Col,
  FormCheck,
  FormGroup,
  Placeholder,
  Row,
} from "react-bootstrap";
import {
  Await,
  useFetcher,
  useParams,
  useLoaderData,
  useRevalidator,
} from "react-router-dom";
import ApiClient, {
  Aggregator,
  NewAggregator,
  formikErrors,
} from "../../ApiClient";
import React, { Suspense } from "react";

import { Check, Clock, CodeSlash } from "react-bootstrap-icons";
import css from "./index.module.css";
import { AggregatorForm } from "../../aggregators/AggregatorForm";
import { FormikHelpers } from "formik";
import { ApiClientContext } from "../../ApiClientContext";

export default function AggregatorTypeSelection() {
  const { aggregators } = useLoaderData() as {
    aggregators: Promise<Aggregator[]>;
  };

  const [useSharedAggregators, setUseSharedAggregators] = React.useState<
    null | boolean
  >(null);

  const fetcher = useFetcher();
  const { accountId } = useParams() as { accountId: string };
  const [showAggregatorForm, setShowAggregatorForm] =
    React.useState<boolean>(false);

  const next = React.useCallback(() => {
    if (useSharedAggregators) {
      fetcher.submit(
        { intends_to_use_shared_aggregators: true },
        {
          action: `/accounts/${accountId}`,
          method: "PATCH",
          encType: "application/json",
        },
      );
    } else {
      setShowAggregatorForm(true);
    }
  }, [useSharedAggregators, fetcher, accountId]);

  const partnerAggregators = React.useMemo(
    () =>
      aggregators.then((aggregators) =>
        aggregators.filter((a) => !a.is_first_party && a.account_id === null),
      ),
    [aggregators],
  );

  if (showAggregatorForm) {
    return <InlineAggregatorForm />;
  } else {
    return (
      <>
        <h5>
          How do you want Divvi Up to pair with an Aggregator? You can always
          change this later.
        </h5>

        <FormGroup
          className={`border border-${
            useSharedAggregators === true ? "primary" : "secondary"
          } rounded p-3 my-2 ${css.aggregatorSelection}`}
          onClick={() => setUseSharedAggregators(true)}
        >
          <FormCheck
            checked={useSharedAggregators === true}
            type="radio"
            id="shared"
            name="aggregator"
            value="shared"
            label={
              <strong>
                Choose from a trusted partner aggregator{" "}
                <Badge pill bg="success">
                  Easier
                </Badge>
              </strong>
            }
            onChange={() => {
              setUseSharedAggregators(true);
            }}
          />
          <Row>
            <Col lg="8">
              <ul className="d-block">
                <li>Easier setup with less technical expertise needed</li>
                <li>
                  During task setup, you&apos;ll choose one of our partners from
                  a list, and that&apos;s it!
                </li>
                <li>
                  Some of our trusted partners:
                  <ol className="d-block">
                    <Suspense fallback={<Placeholder />}>
                      <Await resolve={partnerAggregators}>
                        {(partnerAggregators: Aggregator[]) => {
                          return partnerAggregators.length === 0 ? (
                            <li className="text-danger">
                              There are no partner aggregators on this server
                            </li>
                          ) : (
                            partnerAggregators.map((a) => (
                              <li key={a.id}>{a.name}</li>
                            ))
                          );
                        }}
                      </Await>
                    </Suspense>
                  </ol>
                </li>
              </ul>
            </Col>
            <Col className="text-end" lg="4">
              <div>
                <Check />{" "}
                <Suspense fallback={<Placeholder width={3} />}>
                  <Await resolve={partnerAggregators}>
                    {(aggregators) => aggregators.length}
                  </Await>
                </Suspense>{" "}
                available
              </div>
              <div>
                <Clock /> Shorter setup time
              </div>
            </Col>
          </Row>
        </FormGroup>
        <FormGroup
          className={`border border-${
            useSharedAggregators === false ? "primary" : "secondary"
          } rounded p-3 my-2 ${css.aggregatorSelection}`}
          onClick={() => setUseSharedAggregators(false)}
        >
          <FormCheck
            checked={useSharedAggregators === false}
            type="radio"
            id="byoa"
            name="aggregator"
            value="private"
            label={
              <strong>
                Pair your own Aggregator{" "}
                <Badge pill bg="warning">
                  Advanced
                </Badge>
              </strong>
            }
            onChange={() => {
              setUseSharedAggregators(false);
            }}
          />
          <Row>
            <Col lg="8">
              <ul className="d-block">
                <li>
                  This allows you to self-host your aggregator, but requires
                  advanced understanding of aggregator integration.
                </li>
              </ul>
            </Col>
            <Col className="text-end" lg="4">
              <div>
                <CodeSlash /> Set up with your internal team
              </div>
              <div>
                <Clock /> Longer setup time
              </div>
            </Col>
          </Row>
        </FormGroup>
        <Button
          disabled={useSharedAggregators === null}
          type="submit"
          onClick={next}
        >
          Next
        </Button>
      </>
    );
  }
}

async function submit(
  apiClient: ApiClient,
  accountId: string,
  newAggregator: NewAggregator,
  actions: FormikHelpers<NewAggregator>,
  setAggregator: (aggregator: Aggregator) => void,
) {
  const aggregator = await apiClient.createAggregator(accountId, newAggregator);

  if ("error" in aggregator) {
    actions.setErrors(formikErrors(aggregator.error));
  } else {
    setAggregator(aggregator);
  }
}

function InlineAggregatorForm() {
  const params = useParams();
  const accountId = params.accountId as string;
  const apiClient = React.useContext(ApiClientContext);
  const [aggregator, setAggregator] = React.useState<Aggregator | null>(null);
  const handleSubmit = React.useCallback(
    (values: NewAggregator, actions: FormikHelpers<NewAggregator>) =>
      submit(apiClient, accountId as string, values, actions, setAggregator),
    [apiClient, accountId, setAggregator],
  );
  const { revalidate, state } = useRevalidator();

  if (aggregator) {
    return (
      <Row>
        <Col>
          <h3>Pair your own aggregator</h3>
          Success
          <Button onClick={revalidate} disabled={state === "loading"}>
            Next
          </Button>
        </Col>
      </Row>
    );
  } else {
    return (
      <Row>
        <Col>
          <h3>Pair your own aggregator</h3>
          <AggregatorForm handleSubmit={handleSubmit} />
        </Col>
      </Row>
    );
  }
}
