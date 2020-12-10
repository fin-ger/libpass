Feature: Using GPG IDs
  Scenario: Setting the GPG ID for a subdirectory of the password store
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a different GPG ID is set for a subdirectory of the password store
    Then a .gpg-id file is created in that subdirectory

  Scenario: Using the next-in-parent-directories GPG ID to decrypt a password
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is opened within a directory with a different GPG ID
    Then the password is decrypted using the GPG ID of its parent directory

  Scenario: Using the next-in-parent-directories GPG ID to encrypt a password
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is created within a directory with a different GPG ID
    Then the password is encrypted using the GPG ID of its parent directory

  Scenario: Decrypt password with overriden GPG ID from the environment
    Given a password store exists at the default location
    And a foreign GPG ID is set via the environment
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password which was encrypted using a foreign GPG ID is opened
    Then the password is decrypted using the foreign GPG ID
    
  Scenario: Encrypt password with overriden GPG ID from the environment
    Given a password store exists at the default location
    And a foreign GPG ID is set via the environment
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is created
    Then the password is encrypted using the foreign GPG ID
