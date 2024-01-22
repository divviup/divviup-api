import FormControl from "react-bootstrap/FormControl";
import FormSelect from "react-bootstrap/FormSelect";
import React from "react";
import { Aggregator, CollectorCredential } from "../../ApiClient";
import { Props, TaskFormGroup } from ".";
import { ShortHelpAndLabel } from "./HelpText";
import { useLoaderPromise } from "../../util";

export default function CollectorCredentialSelect(props: Props) {
  const collectorCredentials = useLoaderPromise<CollectorCredential[]>(
    "collectorCredentials",
    [],
  );
  const aggregators = useLoaderPromise<Aggregator[]>("aggregators", []);
  const leader = React.useMemo(
    () =>
      aggregators.find(({ id }) => id === props.values.leader_aggregator_id),
    [props.values.leader_aggregator_id, aggregators],
  );
  const enabledCredentials = React.useMemo(
    () =>
      leader && leader.features.includes("TokenHash")
        ? collectorCredentials.filter(
            (collectorCredential) => !!collectorCredential.token_hash,
          )
        : collectorCredentials,
    [collectorCredentials, leader],
  );

  const setFieldValue = props.setFieldValue;
  React.useEffect(() => {
    if (enabledCredentials.length === 1) {
      setFieldValue("collector_credential_id", enabledCredentials[0].id);
    }
  }, [enabledCredentials, setFieldValue]);

  return (
    <TaskFormGroup controlId="collector_credential_id">
      <ShortHelpAndLabel
        fieldKey="collector_credential_id"
        setFocusedField={props.setFocusedField}
      />
      <FormSelect
        isInvalid={!!props.errors.collector_credential_id}
        id="collector-credential-id"
        name="collector_credential_id"
        onChange={props.handleChange}
        onBlur={props.handleBlur}
        value={props.values.collector_credential_id || ""}
      >
        <option value=""></option>
        {enabledCredentials.map((collectorCredential) => (
          <option key={collectorCredential.id} value={collectorCredential.id}>
            {collectorCredential.name}
          </option>
        ))}
      </FormSelect>
      <FormControl.Feedback type="invalid">
        {props.errors.collector_credential_id}
      </FormControl.Feedback>
    </TaskFormGroup>
  );
}
