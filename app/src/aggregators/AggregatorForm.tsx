import Breadcrumb from "react-bootstrap/Breadcrumb";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import Button from "react-bootstrap/Button";
import { AccountBreadcrumbs, WithAccount } from "../util";
import { CloudUpload } from "react-bootstrap-icons";
import React from "react";
import { LinkContainer } from "react-router-bootstrap";
import { Formik, FormikErrors, FormikHelpers, FormikProps } from "formik";
import Form from "react-bootstrap/Form";
import FormControl from "react-bootstrap/FormControl";
import FormGroup from "react-bootstrap/FormGroup";
import FormLabel from "react-bootstrap/FormLabel";
import FormSelect from "react-bootstrap/FormSelect";
import ApiClient, { NewAggregator, formikErrors } from "../ApiClient";
import {
  NavigateFunction,
  useActionData,
  useNavigate,
  useNavigation,
  useParams,
} from "react-router-dom";
import { ApiClientContext } from "../ApiClientContext";
const { Suspense } = React;
import Placeholder from "react-bootstrap/Placeholder";

async function submit(
  apiClient: ApiClient,
  accountId: string,
  newAggregator: NewAggregator,
  actions: FormikHelpers<NewAggregator>,
  navigate: NavigateFunction
) {
  try {
    let aggregator = await apiClient.createAggregator(accountId, newAggregator);

    if ("error" in aggregator) {
      actions.setErrors(formikErrors(aggregator.error));
    } else {
      return navigate(`/accounts/${accountId}/aggregators/${aggregator.id}`);
    }
  } catch (e) {
    console.log(e);
  }
}

export function AggregatorForm({
  handleSubmit,
  showIsFirstParty = false,
}: {
  handleSubmit: (
    aggregator: NewAggregator,
    helpers: FormikHelpers<NewAggregator>
  ) => void;
  showIsFirstParty?: boolean;
}) {
  const actionData = useActionData();
  let errors: undefined | FormikErrors<NewAggregator> = undefined;
  if (typeof actionData === "object" && actionData && "error" in actionData) {
    errors = actionData.error as FormikErrors<NewAggregator>;
  }
  const navigation = useNavigation();

  return (
    <Formik
      validateOnChange={false}
      validateOnBlur={false}
      validateOnMount={false}
      errors={errors}
      initialValues={
        {
          name: "",
          api_url: "",
          bearer_token: "",
          is_first_party: showIsFirstParty ? true : undefined,
        } as NewAggregator
      }
      onSubmit={handleSubmit}
    >
      {(props) => (
        <Form
          method="post"
          onSubmit={props.handleSubmit}
          noValidate
          autoComplete="off"
        >
          <Name {...props} />
          <ApiUrl {...props} />
          <BearerToken {...props} />
          {showIsFirstParty ? <IsFirstParty {...props} /> : null}
          <Button
            variant="primary"
            type="submit"
            disabled={navigation.state === "submitting"}
          >
            Submit
          </Button>
        </Form>
      )}
    </Formik>
  );
}

export default function AggregatorFormPage() {
  let params = useParams();
  const accountId = params.account_id as string;
  const navigate = useNavigate();
  const apiClient = React.useContext(ApiClientContext);
  const handleSubmit = React.useCallback(
    (values: NewAggregator, actions: FormikHelpers<NewAggregator>) =>
      submit(apiClient, accountId as string, values, actions, navigate),
    [apiClient, accountId, navigate]
  );

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
      <Row>
        <Col>
          <AggregatorForm handleSubmit={handleSubmit} />
        </Col>
      </Row>
    </>
  );
}

function IsFirstParty(props: FormikProps<NewAggregator>) {
  return (
    <FormGroup>
      <Form.Switch
        id="is_first_party"
        label="First Party?"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        checked={props.values.is_first_party}
      />
    </FormGroup>
  );
}

function Name(props: FormikProps<NewAggregator>) {
  return (
    <FormGroup>
      <FormLabel>Name</FormLabel>

      <FormControl
        type="text"
        name="name"
        autoComplete="off"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        value={props.values.name}
        isInvalid={!!props.errors.name}
        data-1p-ignore
      />
      <FormControl.Feedback type="invalid">
        {props.errors.name}
      </FormControl.Feedback>
    </FormGroup>
  );
}

function BearerToken(props: FormikProps<NewAggregator>) {
  return (
    <FormGroup>
      <FormLabel>Bearer Token</FormLabel>

      <FormControl
        type="text"
        name="bearer_token"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        autoComplete="off"
        value={props.values.bearer_token}
        isInvalid={!!props.errors.bearer_token}
      />
      <FormControl.Feedback type="invalid">
        {props.errors.bearer_token}
      </FormControl.Feedback>
    </FormGroup>
  );
}

function ApiUrl(props: FormikProps<NewAggregator>) {
  return (
    <FormGroup>
      <FormLabel>API Url</FormLabel>

      <FormControl
        type="url"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        value={props.values.api_url}
        isInvalid={!!props.errors.api_url}
        autoComplete="off"
        name="api_url"
        placeholder="https://example.com"
        pattern="https://.*"
        required
      />
      <FormControl.Feedback type="invalid">
        {props.errors.api_url}
      </FormControl.Feedback>
    </FormGroup>
  );
}

function Breadcrumbs() {
  return (
    <AccountBreadcrumbs>
      <LinkContainer to="..">
        <Breadcrumb.Item>Aggregators</Breadcrumb.Item>
      </LinkContainer>
      <Breadcrumb.Item active>New</Breadcrumb.Item>
    </AccountBreadcrumbs>
  );
}
