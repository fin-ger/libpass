Feature: Incompatibilities with the upstream pass implementation
  Scenario: Produce an error if the password store directory is overridden over the environment
    Given a password store exists
    And a the password store directory is set in the environment
    And a password store is opened
    When the password store is successfully opened
    Then a store error is emitted that the overridden password store directory from the environment is a security risk

  Scenario: Produce an error if the password store key is overridden over the environment
    Given a password store exists
    And a password store key is set in the environment
    And a password store is opened
    When the password store is successfully opened
    Then a store error is emitted that the overriden password store key from the environment is a security risk

  Scenario: Do not support setting the generated password length over the environment
    Given a password store exists
    And the generated password length is set in the environment
    And a password store is opened
    When the password store is successfully opened
    Then a store error is emitted that the generated password length from the environment is ingored due to security reasons

  Scenario: Do not support setting the character set over the environment
    Given a password store exists
    And the character set is set in the environment
    And a password store is opened
    When the password store is successfully opened
    Then a store error is emitted that the character set from the environment is ingored due to security reasons

  Scenario: Do not support setting the ignored symbols over the environment
    Given a password store exists
    And the ignored symbols is set in the environment
    And a password store is opened
    When the password store is successfully opened
    Then a store error is emitted that the ignored symbols from the environment is ingored due to security reasons

  Scenario: Do not support setting GPG options over the environment
    Given a password store exists
    And GPG options are set in the environment
    And a password store is opened
    When the password store is successfully opened
    Then a store error is emitted that GPG options from the environment are ingored due to security reasons

  Scenario: Produce an error if the environment suggests that pass extensions are enabled
    Given a password store exists
    And pass extensions are enabled in the environment
    And a password store is opened
    When the password store is successfully opened
    Then a store error is emitted that extensions are not supported

  Scenario: Produce an error if the pass extensions directory is set over the environment
    Given a password store exists
    And the pass extensions directory is set over the environment
    And a password store is opened
    When the password store is successfully opened
    Then a store error is emitted that extension directories are ignored as extensions are not supported
