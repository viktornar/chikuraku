import React from "react";
import { Switch, Route } from "react-router-dom";
import { compose } from "recompose";
// Constants
import { SIGN_IN, HOME } from "../constants";
// Hocs
import { withAuthorization } from "../session";
// Pages
import SignIn from "../signIn";
import Chat from "../chat";
// Main store
import store from "./store";

function App() {
  return (
    <Switch>
      <Route path={SIGN_IN} exact component={SignIn} />
      <Route path={HOME} exact component={Chat} />
      <Route component={() => <div>Not found</div>} />
    </Switch>
  );
}

export default compose(withAuthorization)(App);
export { store };
