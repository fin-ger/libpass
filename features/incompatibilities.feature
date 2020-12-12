Feature: Incompatibilities with the upstream pass implementation
  Scenario: Do not support setting the generated password length over the environment
    Given a password store exists at the default location
    And the generated password length is set in the environment
    And a password store is opened at the default location
    When the password store is successfully opened
    Then a store error is emitted that the generated password length from the environment is ingored due to security reasons

  Scenario: Do not support setting the character set over the environment
    Given a password store exists at the default location
    And the character set is set in the environment
    And a password store is opened at the default location
    When the password store is successfully opened
    Then a store error is emitted that the character set from the environment is ingored due to security reasons

  Scenario: Do not support setting the ignored symbols over the environment
    Given a password store exists at the default location
    And the ignored symbols is set in the environment
    And a password store is opened at the default location
    When the password store is successfully opened
    Then a store error is emitted that the ignored symbols from the environment is ingored due to security reasons

  Scenario: Do not support setting GPG options over the environment
    Given a password store exists at the default location
    And GPG options are set in the environment
    And a password store is opened at the default location
    When the password store is successfully opened
    Then a store error is emitted that GPG options from the environment are ingored due to security reasons

  Scenario: Produce an error if the environment suggests that pass extensions are enabled
    Given a password store exists at the default location
    And pass extensions are enabled in the environment
    And a password store is opened at the default location
    When the password store is successfully opened
    Then a store error is emitted that extensions are not supported

  Scenario: Produce an error if the pass extensions directory is set over the environment
    Given a password store exists at the default location
    And the pass extensions directory is set over the environment
    And a password store is opened at the default location
    When the password store is successfully opened
    Then a store error is emitted that extension directories are ignored as extensions are not supported
