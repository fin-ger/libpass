Feature: Using GPG IDs
  Scenario: Setting the GPG ID for a subdirectory of the password store
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a different GPG ID is set for a subdirectory of the password store
    Then a .gpg-id file is created in that subdirectory

  Scenario: Using the next-in-parent-directories GPG ID to decrypt a password
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is opened within a directory with a different GPG ID
    Then the password is decrypted using the GPG ID of its parent directory

  Scenario: Using the next-in-parent-directories GPG ID to encrypt a password
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And a password is created within a directory with a different GPG ID
    Then the password is encrypted using the GPG ID of its parent directory

  Scenario: Decrypt password with overriden GPG ID from the environment
    Given a password store exists
    And a foreign GPG ID is set via the environment
    And a password store is opened
    When the password store is successfully opened
    And a password which was encrypted using a foreign GPG ID is opened
    Then the password is decrypted using the foreign GPG ID
    
  Scenario: Encrypt password with overriden GPG ID from the environment
    Given a password store exists
    And a foreign GPG ID is set via the environment
    And a password store is opened
    When the password store is successfully opened
    And a password is created
    Then the password is encrypted using the foreign GPG ID

  Scenario: GPG IDs are signed if a signing key is manually specified
    Given a password store exists
    And a signing key is manually specified
    And a password store is opened
    When the password store is successfully opened
    Then the GPG IDs in the password store are signed

  Scenario: GPG IDs are not signed no signing key is specified
    Given a password store exists
    And no signing key is specified
    And automatic signing key detection is used
    And a password store is opened
    Then the GPG IDs in the password store are not signed
    And a store error is emitted that a missing signing key is insecure

  Scenario: GPG IDs are signed if a signing key is specified over the environment
    Given a password store exists
    And a signing key is specified in the environment
    And automatic signing key detection is used
    And a password store is opened
    When the password store is successfully opened
    Then the GPG IDs in the password store are signed
