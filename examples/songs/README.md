# REVErse songs

## General

This folder contains many song for reversing some parts of the file
format.

So we have:

 * `DEFAULT.m8s` : old default song
 * `V4EMPTY.m8s` : Empty song in the 4.0 format
 * `V4-1EMPTY.m8s` : Empty song in the 4.1 format (different form V4EMPTY.m8s)
 * `TEST-FILE.m8s` : Historic test song

## CommandMappingV4

So we have a literal test song named `CMDMAPPING.m8s`
that try to list all parameters and instructions along with
screenshot of the various configuration of instruments/phrase/song.

With this it was relatively straightforward to map all the missing
commands. This is expected to be a reference for reverse engineering
the file format.

As much as possible the filename include the instrument/eq number/phrase
for ease of mapping.

 * CMDMAPPING_4_0.m8s : first version of the file, saved with a firmware v4
 * CMDMAPPING_4_1_beta.m8s : same file as before, but saved with
   a firmware 4.1. There is differences :).

