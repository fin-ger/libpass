Feature: General password store operations
  Scenario: Setting a custom passphrase provider when creating a password store
    Given a password store exists
    And passwords are stored in the password store
    And a passphrase provider is available
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    Then the passphrase can be read

  Scenario: Using the system agent to unlock passwords when creating a password store
    Given a password store exists
    And passwords are stored in the password store
    And the system agent is used to unlock passwords
    And a password store is opened
    When the password store is successfully opened
    And a password is opened
    Then the passphrase can be read

  Scenario: Using the system agent to unlock passwords on an existing password store
    Given a password store exists
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And the system agent is set to unlock passwords
    And a password is opened
    Then the passphrase can be read

  Scenario: Searching for a password in the password store succeeds
    Given a password store exists
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And an existing password is searched in the password store
    Then the password is found

  Scenario: Searching for a password in the password store fails
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a non-existent password is searched in the password store
    Then the password is not found

  Scenario: Searching for password content in the password store succeeds
    Given a password store exists
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And content of an existing password is searched in the password store
    Then the password is found

  Scenario: Searching for password content in the password store fails
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And content of a non-existing password is searched in the password store
    Then the password is not found

  Scenario: New files in the password store should respect the automatically detected umask
    Given a password store exists
    And the password store umask is automatically detected
    And a password store is opened
    When the password store is successfully opened
    And a new password is created
    Then the new password respects umask 077

  Scenario: New files in the password store should respect a manually provided umask
    Given a password store exists
    And the password store umask is manually set to 027
    And a password store is opened
    When the password store is successfully opened
    And a new password is created
    Then the new password respects umask 027

  Scenario: New files in the password store should respect an umask from the environment
    Given a password store exists
    And the password store umask environment variable is set to 027
    And the password store umask is automatically detected
    And a password store is opened
    When the password store is successfully opened
    And a new password is created
    Then the new password respects umask 027
