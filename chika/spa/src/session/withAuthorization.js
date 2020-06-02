import React from "react";
import { withRouter } from "react-router-dom";
import { compose } from "recompose";

import { SIGN_IN, HOME } from "../constants";

const withAuthorization = (Component) => {
  class WithAuthorization extends React.Component {
    constructor(props) {
      super(props);

      this.state = {
        token: localStorage.getItem("token"),
      };
    }

    componentDidMount() {
      const { token } = this.state;
      const { history, location } = this.props;
      if (!token) {
        history.push(SIGN_IN);
      } else if (location.pathname === SIGN_IN) {
        history.push(HOME);
      }
    }

    render() {
      return <Component {...this.props} />;
    }
  }

  WithAuthorization.propTypes = {};

  return compose(withRouter)(WithAuthorization);
};

export default withAuthorization;
