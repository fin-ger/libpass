Feature: Git operations in the password store
  Scenario: Commit a new password
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a new password is created
    Then the repository is clean and contains a new commit

  Scenario: Commit an edited password
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a password is edited
    Then the repository is clean and contains a new commit

  Scenario: Commit a removed password
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a password is removed
    Then the repository is clean and contains a new commit

  Scenario: Commit a renamed password
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a password is renamed
    Then the repository is clean and contains a new commit

  Scenario: Commit a password in a new directory
    Given a password store exists
    And the password store uses git
    And a password store is opened
    When the password store is successfully opened
    And a directory is created
    And a password is created in the new directory
    Then the repository is clean and contains a new commit

  Scenario: Commit a renamed directory
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a directory is renamed
    Then the repository is clean and contains a new commit

  Scenario: Commit a removed directory
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a directory is removed
    Then the repository is clean and contains a new commit

  Scenario: Query the status of an unaltered password store
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    Then the repository is clean

  Scenario: Push fast-forward commits to the git remote
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And a password store is opened
    When the password store is successfully opened
    And a password is created
    And the commit is pushed to the remote
    Then pushing the commit succeeds

  Scenario: Push of non-fastforward commits to the git remote fails
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And the repository's remote contains new commits
    And a password store is opened
    When the password store is successfully opened
    And a password is created
    And the commit is pushed to the remote
    Then pushing the commit fails

  Scenario: Pull fast-forward changes from git remote without interaction
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And the repository's remote contains new commits
    And a password store is opened
    When the password store is successfully opened
    And changes are pulled from the remote
    Then no conflicts need to be resolved
    And the remote's commits are fast-forwarded
    And the repository is clean

  Scenario: Pull non-fast-forward changes from the git remote with automatic merging
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And the repository's remote contains new commits
    And a password store is opened
    When the password store is successfully opened
    And a new password is created
    And changes are pulled from the remote
    Then no conflicts need to be resolved
    And the remote's commits are merged
    And the repository is clean

  Scenario: Pull non-fast-forward changes from the git remote with manual merging and resolve merge conflict by letting the user resolve it using decrypted passwords
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And the repository's remote contains new commits
    And a password store is opened
    When the password store is successfully opened
    And a password is edited
    And changes are pulled from the remote
    Then merge conflicts are manually resolved
    And the remote's commits are merged
    And the repository is clean

  Scenario: Pull non-fast-forward changes from the git remote with manual merging and resolve merge conflict by letting the user resolve a binary conflict
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And the repository's remote contains new commits of a binary file
    And a password store is opened
    When the password store is successfully opened
    And the binary file is edited
    And changes are pulled from the remote
    Then binary merge conflicts are manually resolved
    And the remote's commits are merged
    And the repository is clean

  Scenario: Pull non-fast-forward changes from the git remote with manual merging and resolve merge conflict by letting the user resolve a plain text conflict
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And the repository's remote contains new commits of a text file
    And a password store is opened
    When the password store is successfully opened
    And the text file is edited
    And changes are pulled from the remote
    Then plain text merge conflicts are manually resolved
    And the remote's commits are merged
    And the repository is clean

  Scenario: Pull non-fast-forward changes from the git remote with manual merging and resolve merge conflict by letting the user resolve a gpg-id conflict
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And the repository's remote contains new commits changing the gpg-id
    And a password store is opened
    When the password store is successfully opened
    And the gpg-id of the store is edited
    And changes are pulled from the remote
    Then gpg-id merge conflicts are manually resolved
    And the remote's commits are merged
    And the repository is clean

  Scenario: Config is invalid when no git username is set
    Given a password store exists
    And the password store uses git
    And the git username is not set
    And a password store is opened
    When the password store is successfully opened
    Then the git config is invalid

  Scenario: Config is invalid when no git email is set
    Given a password store exists
    And the password store uses git
    And the git email is not set
    And a password store is opened
    When the password store is successfully opened
    Then the git config is invalid

  Scenario: Config is valid when git username and email are set
    Given a password store exists
    And the password store uses git
    And a password store is opened
    When the password store is successfully opened
    Then the git config is valid

  Scenario: Git username overridden in repository config
    Given a password store exists
    And the password store uses git
    And a password store is opened
    When the password store is successfully opened
    And the username is overridden in the git config
    Then the git username for this repository is changed

  Scenario: Git email overridden in repository config
    Given a password store exists
    And the password store uses git
    And a password store is opened
    When the password store is successfully opened
    And the email is overridden in the git config
    Then the git email for this repository is changed

  Scenario: Git username can be read from repository config
    Given a password store exists
    And the password store uses git
    And a password store is opened
    When the password store is successfully opened
    Then the git username can be read from the repository config

  Scenario: Git email can be read from repository config
    Given a password store exists
    And the password store uses git
    And a password store is opened
    When the password store is successfully opened
    Then the git email can be read from the repository config

  Scenario: Status of unchanged git repository is clean
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And a password store is opened
    When the password store is successfully opened
    Then the git status is clean

  Scenario: Status of changed git repository contains commits ahead of remote
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And a password store is opened
    When the password store is successfully opened
    And a password is edited
    Then the git status contains new commits

  Scenario: Status of updated remote contains commits ahead of local branch
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And the repository's remote contains new commits
    And a password store is opened
    When the password store is successfully opened
    And the repository's remote is fetched
    Then the git status contains new commits on the remote

  Scenario: Uncommitted files changed outside of this library are reported in status
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And a password store is opened
    When the password store is successfully opened
    And a file in the repository is changed outside of this library
    Then the git status contains uncommitted changes

  Scenario: Files changed outside of this library can be committed
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And a password store is opened
    When the password store is successfully opened
    And a file in the repository is changed outside of this library
    And this file is committed with this library
    Then the git status contains new commits
