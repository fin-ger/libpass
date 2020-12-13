Feature: Git operation in the password store
  Scenario: Commit a new password
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is created
    And the password is committed
    Then the repository is clean and contains a new commit

  Scenario: Commit an edited password
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is edited
    And the password is committed
    Then the repository is clean and contains a new commit

  Scenario: Commit a removed password
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is removed
    And the removal is committed
    Then the repository is clean and contains a new commit

  Scenario: Commit a renamed password
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is renamed
    And the renaming is committed
    Then the repository is clean and contains a new commit

  Scenario: Commit a new directory
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a directory is created
    And the directory is committed
    Then the repository is clean and contains a new commit

  Scenario: Commit a renamed directory
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a directory is renamed
    And the directory is committed
    Then the repository is clean and contains a new commit

  Scenario: Commit a removed directory
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a directory is removed
    And the removal is committed
    Then the repository is clean and contains a new commit

  Scenario: Query the status of an unaltered password store
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    Then the repository is clean

  Scenario: Query the status of an altered password store
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is created
    Then the repository is not clean

  Scenario: Push fast-forward commits to the git remote
    Given a password store exists at the default location
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is created
    And the password is committed
    And the commit is pushed to the remote
    Then pushing the commit succeeds

  Scenario: Push of non-fastforward commits to the git remote fails
    Given a password store exists at the default location
    And the repository's remote contains new commits
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is created
    And the password is committed
    And the commit is pushed to the remote
    Then pushing the commit fails

  Scenario: Pull fast-forward changes from git remote without interaction
    Given a password store exists at the default location
    And the repository's remote contains new commits
    And a password store is opened at the default location
    When the password store is successfully opened
    And changes are pulled from the remote
    Then the repository is clean

  Scenario: Pull non-fast-forward changes from the git remote with automatic merging
    Given a password store exists at the default location
    And the repository's remote contains new commits
    And a password store is opened at the default location
    When the password store is successfully opened
    And a new password is created
    And the password is committed
    And changes are pulled from the remote
    Then the repository is clean

  Scenario: Pull non-fast-forward changes from the git remote with manual merging and resolve merge conflict by letting the user resolve it using decrypted passwords
    Given a password store exists at the default location
    And the repository's remote contains new commits
    And a password store is opened at the default location
    When the password store is successfully opened
    And a password is edited
    And the password is committed
    And changes are pulled from the remote
    Then the merge conflict can be manually resolved
    And the repository is clean
