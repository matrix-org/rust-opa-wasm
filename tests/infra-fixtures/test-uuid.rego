#  Copyright 2022 The Matrix.org Foundation C.I.C.
# 
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
# 
#      http://www.apache.org/licenses/LICENSE-2.0
# 
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.

package test

policy[data] {
	data := {
		# these two must be equal during a single query, but different from one invocation to another
		"one": uuid.rfc4122("id"),
		"two": uuid.rfc4122("id"),
	}
}
