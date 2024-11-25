import { FormikHelpers } from "formik";
import ApiClient, { NewAggregator, formikErrors } from "../ApiClient";
import { AggregatorForm } from "../aggregators/AggregatorForm";
import { useRevalidator } from "react-router";
import { ApiClientContext } from "../ApiClientContext";
import React from "react";

async function submit(
  apiClient: ApiClient,
  newAggregator: NewAggregator,
  actions: FormikHelpers<NewAggregator>,
  revalidate: () => void,
) {
  const aggregator = await apiClient.createSharedAggregator(newAggregator);

  if ("error" in aggregator) {
    actions.setErrors(formikErrors(aggregator.error));
  } else {
    actions.resetForm();
    revalidate();
  }
}

export default function SharedAggreatorForm() {
  const apiClient = React.useContext(ApiClientContext);
  const { revalidate } = useRevalidator();
  const handleSubmit = React.useCallback(
    (values: NewAggregator, actions: FormikHelpers<NewAggregator>) =>
      submit(apiClient, values, actions, revalidate),
    [apiClient, revalidate],
  );

  return (
    <>
      <h1>New Shared Aggregator</h1>
      <AggregatorForm handleSubmit={handleSubmit} showIsFirstParty />
    </>
  );
}
