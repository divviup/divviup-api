import {
  Await,
  useNavigate,
  useNavigation,
  useParams,
  useRouteLoaderData,
  NavigateFunction,
} from "react-router-dom";
import Breadcrumb from "react-bootstrap/Breadcrumb";
import Button from "react-bootstrap/Button";
import Col from "react-bootstrap/Col";
import Form from "react-bootstrap/Form";
import FormControl from "react-bootstrap/FormControl";
import FormGroup from "react-bootstrap/FormGroup";
import FormLabel from "react-bootstrap/FormLabel";
import FormSelect from "react-bootstrap/FormSelect";
import React, { ChangeEvent, ChangeEventHandler } from "react";
import Row from "react-bootstrap/Row";
import { ApiClientContext } from "./ApiClientContext";
import { LinkContainer } from "react-router-bootstrap";
import ApiClient, { Account, NewTask, formikErrors } from "./ApiClient";
import { Formik, FormikHelpers, FormikProps } from "formik";
import FormCheck from "react-bootstrap/FormCheck";
import { DateTime } from "luxon";

async function submit(
  apiClient: ApiClient,
  accountId: string,
  newTask: NewTask,
  actions: FormikHelpers<NewTask>,
  navigate: NavigateFunction
) {
  try {
    let task = await apiClient.createTask(accountId, newTask);

    if ("error" in task) {
      actions.setErrors(formikErrors(task.error));
    } else {
      return navigate(`/accounts/${accountId}/tasks/${task.id}`);
    }
  } catch (e) {
    console.log(e);
  }
}

export default function TaskForm() {
  const { account_id } = useParams();
  const { account } = useRouteLoaderData("account") as {
    account: Promise<Account>;
  };
  const apiClient = React.useContext(ApiClientContext);
  const navigate = useNavigate();
  const handleSubmit = React.useCallback(
    (values: NewTask, actions: FormikHelpers<NewTask>) =>
      submit(apiClient, account_id as string, values, actions, navigate),
    [apiClient, account_id, navigate]
  );
  const navigation = useNavigation();

  return (
    <>
      <Breadcrumbs account={account} />
      <Row>
        <Col sm="9">
          <Formik
            validateOnChange={false}
            validateOnBlur={false}
            validateOnMount={false}
            initialValues={
              {
                min_batch_size: 100,
                partner_url: "https://divviup.cloudflare.com",
                name: "",
                time_precision_seconds: 3600,
                expiration: null,
                is_leader: true,
                hpke_config: "",
              } as NewTask
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
                <TaskName {...props} />
                <VdafType {...props} />
                <VdafDetails {...props} />
                <MinBatchSize {...props} />
                <HpkeConfig {...props} />
                <IsLeader {...props} />
                <TimePrecisionSeconds {...props} />
                <Expiration {...props} />
                <QueryType {...props} />
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
        </Col>
      </Row>
    </>
  );
}

function IsLeader({ handleChange, values }: FormikProps<NewTask>) {
  return (
    <FormCheck
      type="switch"
      checked={values.is_leader}
      onChange={handleChange}
      name="is_leader"
      label="Leader"
    />
  );
}

function QueryType({
  values,
  setFieldValue,
  errors,
  handleBlur,
}: FormikProps<NewTask>) {
  let { max_batch_size } = values;
  const [fixedSize, setFixedSize] = React.useState(false);

  const checkboxChange = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      let checked = event.target.value === "fixed";
      setFixedSize(checked);
      if (!checked) setFieldValue("max_batch_size", null);
    },
    [setFixedSize, setFieldValue]
  );

  const handleChange = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      if (event.target.value) {
        setFieldValue("max_batch_size", event.target.valueAsNumber);
      } else {
        setFieldValue("max_batch_size", null);
      }
    },
    [setFieldValue]
  );

  return (
    <FormGroup className="mb-3" controlId="querytype">
      <FormCheck
        type="radio"
        name="query-type"
        checked={!fixedSize}
        onChange={checkboxChange}
        label="Time Interval"
        value="time"
      />
      <FormCheck
        type="radio"
        name="query-type"
        checked={fixedSize}
        onChange={checkboxChange}
        label="Fixed Size"
        value="fixed"
      />
      {fixedSize ? (
        <FormControl
          type="number"
          name="max_batch_size"
          value={max_batch_size || 1}
          onChange={handleChange}
          step={1}
          onBlur={handleBlur}
          min={1}
          isInvalid={!!errors.max_batch_size}
        />
      ) : null}
      <FormControl.Feedback type="invalid">
        {errors.max_batch_size}
      </FormControl.Feedback>
    </FormGroup>
  );
}

function HpkeConfig({ setFieldValue }: FormikProps<NewTask>) {
  let reader = React.useMemo(() => {
    let reader = new FileReader();
    reader.addEventListener("load", () => {
      if (typeof reader.result === "string") {
        console.log(reader.result);
        console.log(reader.result.split(",")[1]);
        setFieldValue("hpke_config", reader.result.split(",")[1]);
      }
    });
    return reader;
  }, [setFieldValue]);
  let onChange: ChangeEventHandler<HTMLInputElement> = React.useCallback(
    (event) => {
      let files = event.target.files;
      if (files && files[0]) {
        reader.readAsDataURL(files[0]);
      }
    },
    [reader]
  );

  return (
    <FormGroup className="mb-3">
      <FormLabel>DAP-encoded HPKE file</FormLabel>
      <FormControl type="file" onChange={onChange} />
    </FormGroup>
  );
}

function TaskName(props: FormikProps<NewTask>) {
  return (
    <FormGroup className="mb-3" controlId="name">
      <FormLabel>Task Name</FormLabel>
      <FormControl
        type="text"
        name="name"
        placeholder="Task Name"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        value={props.values.name}
        isInvalid={!!props.errors.name}
      />
      <FormControl.Feedback type="invalid">
        {props.errors.name}
      </FormControl.Feedback>
    </FormGroup>
  );
}

function MinBatchSize(props: FormikProps<NewTask>) {
  return (
    <FormGroup className="mb-3" controlId="min_batch_size">
      <FormLabel>Minimum Batch Size</FormLabel>
      <FormControl
        type="number"
        name="min_batch_size"
        min="300"
        value={props.values.min_batch_size}
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        isInvalid={!!props.errors.min_batch_size}
      />
      <FormControl.Feedback type="invalid">
        {props.errors.min_batch_size}
      </FormControl.Feedback>
    </FormGroup>
  );
}

const seconds = {
  minute: 60,
  hour: 60 * 60,
  day: 60 * 60 * 24,
  week: 60 * 60 * 24 * 7,
};
type Unit = keyof typeof seconds;
function TimePrecisionSeconds(props: FormikProps<NewTask>) {
  let { setFieldValue } = props;
  const [count, setCount] = React.useState(1);
  const [unit, setUnit] = React.useState<Unit>("minute");

  React.useEffect(() => {
    setFieldValue("time_precision_seconds", seconds[unit] * count);
  }, [count, unit, setFieldValue]);

  const changeUnit = React.useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      setUnit(event.target.value as Unit);
    },
    [setUnit]
  );

  const changeCount = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setCount(parseInt(event.target.value, 10));
    },
    [setCount]
  );

  return (
    <FormGroup className="mb-3" controlId="time_precision_seconds">
      <FormLabel column>Time Precision</FormLabel>
      <Row>
        <Col xs="2">
          <FormControl
            type="number"
            value={count}
            onChange={changeCount}
            isInvalid={!!props.errors.time_precision_seconds}
          />
        </Col>
        <Col>
          <FormSelect
            value={unit}
            onChange={changeUnit}
            isInvalid={!!props.errors.time_precision_seconds}
          >
            {Object.keys(seconds).map((unit) => (
              <option key={unit} value={unit}>
                {unit}
                {count === 1 ? "" : "s"}
              </option>
            ))}
          </FormSelect>
          <FormControl.Feedback type="invalid">
            {props.errors.time_precision_seconds}
          </FormControl.Feedback>
        </Col>
      </Row>
    </FormGroup>
  );
}

function VdafType(props: FormikProps<NewTask>) {
  return (
    <FormGroup className="mb-3" controlId="vdaf.type">
      <FormLabel>Function</FormLabel>
      <FormSelect
        value={props.values.vdaf?.type}
        name="vdaf.type"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        isInvalid={typeof props.errors.vdaf === "string"}
      >
        <option></option>
        <option value="sum">sum</option>
        <option value="histogram">histogram</option>
        <option value="count">count</option>
      </FormSelect>
      <FormControl.Feedback type="invalid">
        {typeof props.errors.vdaf === "string" ? props.errors.vdaf : null}
      </FormControl.Feedback>
    </FormGroup>
  );
}

function VdafDetails(props: FormikProps<NewTask>) {
  switch (props.values.vdaf?.type) {
    case "sum":
      return <SumBits {...props} />;

    case "histogram":
      return <HistogramBucketSelection {...props} />;

    default:
      return <></>;
  }
}

function HistogramBucketSelection(props: FormikProps<NewTask>) {
  let { setFieldValue } = props;
  let [input, setInput] = React.useState(
    props.values.vdaf?.type === "histogram"
      ? (props.values.vdaf?.buckets || []).join(", ")
      : ""
  );

  let cb = React.useCallback(
    (change: ChangeEvent<HTMLInputElement>) => {
      if (/^([0-9]+, *)*[0-9]*$/.test(change.target.value)) {
        if (input.length) {
          setFieldValue(
            "vdaf.buckets",
            input
              .split(/, */)
              .map((n) => parseInt(n, 10))
              .filter((n) => !isNaN(n))
          );
        }
        setInput(change.target.value);
      } else {
        change.stopPropagation();
        change.preventDefault();
      }
    },
    [input, setInput, setFieldValue]
  );

  let blur = React.useCallback(() => {
    let value = input
      .split(/, */)
      .map((n) => parseInt(n, 10))
      .sort((a, b) => a - b);
    value = [...new Set(value)];
    setInput(value.join(", "));
    setFieldValue("vdaf.buckets", value);
  }, [input, setInput, setFieldValue]);

  if (props.values.vdaf?.type !== "histogram") return <></>;

  return (
    <FormGroup className="mb-3" controlId="vdaf.bits">
      <FormLabel>Histogram buckets</FormLabel>
      <FormControl
        value={input}
        name="vdaf.buckets"
        onChange={cb}
        onBlur={blur}
        isInvalid={
          typeof props.errors.vdaf === "object" &&
          "buckets" in props.errors.vdaf &&
          !!props.errors.vdaf.buckets
        }
      />
      <FormControl.Feedback type="invalid">
        {typeof props.errors.vdaf === "object" &&
          "buckets" in props.errors.vdaf &&
          props.errors.vdaf.buckets}
      </FormControl.Feedback>
    </FormGroup>
  );
}

function SumBits(props: FormikProps<NewTask>) {
  let { setFieldValue } = props;
  let handleChange = React.useCallback(
    (event: ChangeEvent<HTMLSelectElement>) =>
      setFieldValue("vdaf.bits", parseInt(event.target.value, 10)),
    [setFieldValue]
  );
  if (props.values.vdaf?.type !== "sum") return <></>;

  return (
    <FormGroup className="mb-3" controlId="vdaf.bits">
      <FormLabel>Maximum Sum Value</FormLabel>
      <FormSelect
        value={props.values.vdaf?.bits}
        name="vdaf.bits"
        onChange={handleChange}
        onBlur={props.handleBlur}
      >
        {[...new Array(128)].map((_, i) => (
          <option value={i} key={i}>
            {Math.pow(2, i)}
          </option>
        ))}
      </FormSelect>
    </FormGroup>
  );
}

function Expiration(props: FormikProps<NewTask>) {
  let { setFieldValue, values } = props;
  let { expiration } = values;

  const [enabled, setEnabled] = React.useState(false);

  const checkboxChange = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setEnabled(event.target.checked);
      if (!event.target.checked) setFieldValue("expiration", null);
    },
    [setEnabled, setFieldValue]
  );

  const handleChange = React.useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      if (event.target.value) {
        let datetime = DateTime.fromISO(event.target.value);
        setFieldValue(
          "expiration",
          datetime.toISO({
            suppressSeconds: true,
            suppressMilliseconds: true,
            includeOffset: true,
          })
        );
      } else {
        setFieldValue("expiration", null);
      }
    },
    [setFieldValue]
  );

  let min = React.useMemo(
    () =>
      DateTime.now().set({ second: 0, millisecond: 0 }).toISO({
        includeOffset: false,
        suppressSeconds: true,
        suppressMilliseconds: true,
      }),
    []
  );

  return (
    <FormGroup className="mb-3" controlId="expiration">
      <FormCheck
        type="switch"
        checked={enabled}
        onChange={checkboxChange}
        label="Expiration"
      />
      {enabled ? (
        <FormControl
          type="datetime-local"
          name="expiration"
          value={
            expiration
              ? DateTime.fromISO(expiration)
                .toLocal()
                .set({ second: 0, millisecond: 0 })
                .toISO({
                  includeOffset: false,
                  suppressSeconds: true,
                  suppressMilliseconds: true,
                }) || undefined
              : ""
          }
          onChange={handleChange}
          onBlur={props.handleBlur}
          step={60}
          min={min || undefined}
          isInvalid={!!props.errors.expiration}
        />
      ) : null}
      <FormControl.Feedback type="invalid">
        {props.errors.expiration}
      </FormControl.Feedback>
    </FormGroup>
  );
}

// <FormControl
//   value={props.values.vdaf?.type}
//   name="vdaf.bits"
//   onChange={props.handleChange}
//   onBlur={props.handleBlur}
//   isInvalid={!!props.errors.time_precision_seconds}
// />

function Breadcrumbs({ account }: { account: Promise<Account> }) {
  let { account_id } = useParams();

  return (
    <Row>
      <Col>
        <Breadcrumb>
          <LinkContainer to="/">
            <Breadcrumb.Item>Home</Breadcrumb.Item>
          </LinkContainer>
          <LinkContainer to="/accounts">
            <Breadcrumb.Item>Accounts</Breadcrumb.Item>
          </LinkContainer>
          <LinkContainer to={`/accounts/${account_id}`}>
            <Breadcrumb.Item>
              <React.Suspense fallback={<span>...</span>}>
                <Await resolve={account}>{(account) => account.name}</Await>
              </React.Suspense>
            </Breadcrumb.Item>
          </LinkContainer>

          <LinkContainer to={`/accounts/${account_id}/tasks`}>
            <Breadcrumb.Item>Tasks</Breadcrumb.Item>
          </LinkContainer>
          <Breadcrumb.Item active>new</Breadcrumb.Item>
        </Breadcrumb>
      </Col>
    </Row>
  );
}
